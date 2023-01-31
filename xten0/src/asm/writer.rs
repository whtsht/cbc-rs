use super::inst::{WriteInst, WriteInstExt};
use super::obj::*;
use super::operand::{memory, Gpr64, Rip, Xmm};
use derive_new::new;
use std::collections::BTreeMap;
use std::io::{self, Cursor, Write};

/// Writer for building an object.
#[derive(Debug, Clone)]
pub struct Writer {
    text: Cursor<Vec<u8>>,
    data: Cursor<Vec<u8>>,
    rodata: Cursor<Vec<u8>>,
    bss: u64,
    defs: Vec<Option<Def>>, // Indexed by label.index
    uses: Vec<Use>,
    names: BTreeMap<String, Label>,
}

impl Writer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            text: Cursor::new(Vec::new()),
            data: Cursor::new(Vec::new()),
            rodata: Cursor::new(Vec::new()),
            bss: 0,
            defs: Vec::new(),
            uses: Vec::new(),
            names: BTreeMap::new(),
        }
    }

    /// Issue an unnamed label.
    pub fn issue_label(&mut self) -> Label {
        let label = Label::new(self.defs.len());
        self.defs.push(None);
        label
    }

    /// Get a named label.
    pub fn get_label(&mut self, name: &str) -> Label {
        if let Some(&label) = self.names.get(name) {
            label
        } else {
            let label = self.issue_label();
            self.names.insert(name.to_string(), label);
            label
        }
    }

    /// Use the specified label at the location.
    pub fn r#use(&mut self, location: Location, label: Label, addend: i64, ty: RelocType) {
        self.uses.push(Use::new(location, label, addend, ty));
    }

    pub fn text(&mut self) -> TextWriter {
        TextWriter(self)
    }

    pub fn data(&mut self) -> DataWriter {
        DataWriter(self)
    }

    pub fn rodata(&mut self) -> RodataWriter {
        RodataWriter(self)
    }

    pub fn bss(&mut self) -> BssWriter {
        BssWriter(self)
    }

    /// Produce an object by resolving definitions and uses.
    pub fn produce(mut self) -> io::Result<Object> {
        let mut label_symbols = BTreeMap::new(); // label: Label -> symbol.name: String

        // Named labels remain on the object as symbols.
        let symbols = self
            .names
            .into_iter()
            .map(|(name, label)| {
                let def = self.defs[label.index];
                let binding = match def {
                    Some(def) if !def.is_global => Binding::Local(def.location),
                    Some(def) => Binding::Global(Some(def.location)),
                    None => Binding::Global(None),
                };
                label_symbols.insert(label, name.clone());
                Symbol::new(name, binding)
            })
            .collect::<Vec<_>>();

        // Uses that cannot be resolved at this time become relocations.
        let mut relocs = Vec::new();
        for u in self.uses {
            let mut target = None;
            let mut offset = 0;

            if let Some(d) = self.defs[u.target.index] {
                // If the definition is within the object, try to resolve the use with the definition.
                use LocationSection::*;
                if let Some(buf) = match (d.location.section, u.location.section) {
                    (Text, Text) => Some(&mut self.text),
                    (Data, Data) => Some(&mut self.data),
                    (Rodata, Rodata) => Some(&mut self.rodata),
                    _ => None,
                } {
                    if u.ty
                        .resolve_statically(buf, d.location.pos, u.location.pos, u.addend)?
                    {
                        continue;
                    }
                }

                // If failed, we need a relocation for the use.
                target = Some(RelocTarget::Section(d.location.section));
                offset = d.location.pos as i64;
            }

            if let Some(symbol) = label_symbols.get(&u.target) {
                // Prefer RelocTarget::Symbol to RelocTarget::Section.
                target = Some(RelocTarget::Symbol(symbol.clone()));
                offset = 0;
            }

            if let Some(target) = target {
                relocs.push(Reloc::new(u.location, target, offset + u.addend, u.ty));
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Cannot resolve label={}: There is no symbol or definition corresponding to the label.", u.target.index),
                ))?;
            }
        }

        Ok(Object {
            text: self.text.into_inner(),
            data: self.data.into_inner(),
            rodata: self.rodata.into_inner(),
            bss: self.bss,
            symbols,
            relocs,
        })
    }
}

/// On the Writer, a specific position on an object is identified as a label.
///
/// Labels may or may not be named. Named labels remain on the object as symbols.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash, new)]
pub struct Label {
    index: usize,
}

/// A definition associated with a label. A label may or may not be associated with a definition.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, new)]
struct Def {
    is_global: bool,
    location: Location,
}

/// A label use. Uses that cannot be resolved at object production time become relocations.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, new)]
struct Use {
    location: Location,
    target: Label,
    addend: i64,
    ty: RelocType,
}

pub trait SectionWrite {
    /// Get the writer of this section.
    fn writer(&mut self) -> &mut Writer;

    /// Get the current location.
    fn location(&self) -> Location;

    /// Start the definition to be associated with `label`.
    fn define(&mut self, label: Label, is_global: bool) -> Location {
        assert!(
            self.writer().defs[label.index].is_none(),
            "Duplicate definition for label={}",
            label.index
        );
        let location = self.location();
        self.writer().defs[label.index] = Some(Def::new(is_global, location));
        location
    }

    /// Use the specified label at the location from `offset_from_current`.
    fn use_relative(&mut self, offset_from_current: i64, label: Label, addend: i64, ty: RelocType)
    where
        Self: io::Write,
    {
        let location = self.location().offset(offset_from_current);
        self.writer().r#use(location, label, addend, ty)
    }

    /// Adjust alignment by writing invalid values.
    fn align(&mut self, align: u64) -> io::Result<()> {
        let current = self.location().pos;
        if current % align != 0 {
            self.allocate(align - current % align)?;
        }
        Ok(())
    }

    /// Allocate the specified space on this section.
    fn allocate(&mut self, size: u64) -> io::Result<()>;
}

/// Writer behaves like TextWriter

impl io::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.text().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.text().flush()
    }
}

impl SectionWrite for Writer {
    fn writer(&mut self) -> &mut Writer {
        self
    }

    fn location(&self) -> Location {
        // NOTE: We cannot use self.text().location() since it requires &mut self
        Location::new(LocationSection::Text, self.text.position())
    }

    fn allocate(&mut self, size: u64) -> io::Result<()> {
        self.text().allocate(size)
    }
}

macro_rules! section_writer {
    ($name:ident, $field:ident, $section:path, $zero:expr) => {
        pub struct $name<'a>(&'a mut Writer);

        impl<'a> io::Write for $name<'a> {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.0.$field.write(buf)
            }

            fn flush(&mut self) -> io::Result<()> {
                self.0.$field.flush()
            }
        }

        impl<'a> SectionWrite for $name<'a> {
            fn writer(&mut self) -> &mut Writer {
                self.0
            }

            fn location(&self) -> Location {
                Location::new($section, self.0.$field.position())
            }

            fn allocate(&mut self, size: u64) -> io::Result<()> {
                for _ in 0..size {
                    self.write_all(&[$zero])?;
                }
                Ok(())
            }
        }
    };
}

section_writer!(TextWriter, text, LocationSection::Text, 0x90); // nop
section_writer!(DataWriter, data, LocationSection::Data, 0);
section_writer!(RodataWriter, rodata, LocationSection::Rodata, 0);

/// Writer for bss sections.
#[derive(Debug)]
pub struct BssWriter<'a>(&'a mut Writer);

impl<'a> SectionWrite for BssWriter<'a> {
    fn writer(&mut self) -> &mut Writer {
        self.0
    }

    fn location(&self) -> Location {
        Location::new(LocationSection::Bss, self.0.bss)
    }

    fn allocate(&mut self, size: u64) -> io::Result<()> {
        self.0.bss += size;
        Ok(())
    }
}

// * In this implementation, we use `RelocType::PcRel32` for label uses by default.
// * We use `RelocType::PcRel8` for jmp instructions if the operand is wrapped by `Short`.
// * We use `RelocType::PcRelToAddressTable32` for call or mov instruction if the operand is wrapped
//   by `AddressTable`. This may be necessary for linking shared objects.

/// Wrapper type for address table, used to force `RelocType::PcRelToAddressTable32` relocation
/// for call, jmp, or mov instructions.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct AddressTable<T>(pub T);

/// Wrapper type for short jmp, used to force `RelocType::PcRel8` relocation for jmp instructions.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Short<T>(pub T);

/// callq Label
impl<W> WriteInst<W> for super::inst::Callq<Label>
where
    W: Write + SectionWrite,
{
    fn write_inst(&self, w: &mut W) -> io::Result<()> {
        w.callq(0i32)?;
        w.use_relative(-4, self.0, -4, RelocType::PcRel32);
        Ok(())
    }
}

/// callq AddressTable<Label>
impl<W> WriteInst<W> for super::inst::Callq<AddressTable<Label>>
where
    W: Write + SectionWrite,
{
    fn write_inst(&self, w: &mut W) -> io::Result<()> {
        w.callq(memory(Rip + 0i32))?;
        w.use_relative(-4, self.0 .0, -4, RelocType::PcRelToAddressTable32);
        Ok(())
    }
}

/// jmpq Label
impl<W> WriteInst<W> for super::inst::Jmpq<Label>
where
    W: Write + SectionWrite,
{
    fn write_inst(&self, w: &mut W) -> io::Result<()> {
        w.jmpq(0i32)?;
        w.use_relative(-4, self.0, -4, RelocType::PcRel32);
        Ok(())
    }
}

/// jmpq AddressTable<Label>
impl<W> WriteInst<W> for super::inst::Jmpq<AddressTable<Label>>
where
    W: Write + SectionWrite,
{
    fn write_inst(&self, w: &mut W) -> io::Result<()> {
        w.jmpq(memory(Rip + 0i32))?;
        w.use_relative(-4, self.0 .0, -4, RelocType::PcRelToAddressTable32);
        Ok(())
    }
}

/// jmpq Short<Label>
impl<W> WriteInst<W> for super::inst::Jmpq<Short<Label>>
where
    W: Write + SectionWrite,
{
    fn write_inst(&self, w: &mut W) -> io::Result<()> {
        w.jmpq(0i8)?;
        w.use_relative(-1, self.0 .0, -1, RelocType::PcRel8);
        Ok(())
    }
}

macro_rules! impl_conditional_jmp {
    ($op:tt::$method:tt) => {
        // jCC Label
        impl<W> WriteInst<W> for super::inst::$op<Label>
        where
            W: Write + SectionWrite,
        {
            fn write_inst(&self, w: &mut W) -> io::Result<()> {
                w.$method(0i32)?;
                w.use_relative(-4, self.0, -4, RelocType::PcRel32);
                Ok(())
            }
        }

        // jCC Short<Label>
        impl<W> WriteInst<W> for super::inst::$op<Short<Label>>
        where
            W: Write + SectionWrite,
        {
            fn write_inst(&self, w: &mut W) -> io::Result<()> {
                w.$method(0i8)?;
                w.use_relative(-1, self.0 .0, -1, RelocType::PcRel8);
                Ok(())
            }
        }
    };
}

impl_conditional_jmp!(Ja::ja);
impl_conditional_jmp!(Jae::jae);
impl_conditional_jmp!(Jb::jb);
impl_conditional_jmp!(Jbe::jbe);
impl_conditional_jmp!(Jc::jc);
impl_conditional_jmp!(Je::je);
impl_conditional_jmp!(Jg::jg);
impl_conditional_jmp!(Jge::jge);
impl_conditional_jmp!(Jl::jl);
impl_conditional_jmp!(Jle::jle);
impl_conditional_jmp!(Jna::jna);
impl_conditional_jmp!(Jnae::jnae);
impl_conditional_jmp!(Jnb::jnb);
impl_conditional_jmp!(Jnbe::jnbe);
impl_conditional_jmp!(Jnc::jnc);
impl_conditional_jmp!(Jne::jne);
impl_conditional_jmp!(Jng::jng);
impl_conditional_jmp!(Jnge::jnge);
impl_conditional_jmp!(Jnl::jnl);
impl_conditional_jmp!(Jnle::jnle);
impl_conditional_jmp!(Jno::jno);
impl_conditional_jmp!(Jnp::jnp);
impl_conditional_jmp!(Jns::jns);
impl_conditional_jmp!(Jnz::jnz);
impl_conditional_jmp!(Jo::jo);
impl_conditional_jmp!(Jp::jp);
impl_conditional_jmp!(Jpe::jpe);
impl_conditional_jmp!(Jpo::jpo);
impl_conditional_jmp!(Js::js);
impl_conditional_jmp!(Jz::jz);

macro_rules! impl_movq_label_o {
    ($o:tt) => {
        /// movq Label O
        impl<W> WriteInst<W> for super::inst::Movq<Label, $o>
        where
            W: Write + SectionWrite,
        {
            fn write_inst(&self, w: &mut W) -> io::Result<()> {
                w.movq(memory(Rip + 0i32), self.1)?;
                w.use_relative(-4, self.0, -4, RelocType::PcRel32);
                Ok(())
            }
        }
    };
}

impl_movq_label_o!(Gpr64);
impl_movq_label_o!(Xmm);
impl_movq_label_o!(i32);

macro_rules! impl_movq_o_label {
    ($o:tt) => {
        /// movq O Label
        impl<W> WriteInst<W> for super::inst::Movq<$o, Label>
        where
            W: Write + SectionWrite,
        {
            fn write_inst(&self, w: &mut W) -> io::Result<()> {
                w.movq(self.0, memory(Rip + 0i32))?;
                w.use_relative(-4, self.1, -4, RelocType::PcRel32);
                Ok(())
            }
        }

        /// movq O AddressTable<Label>
        impl<W> WriteInst<W> for super::inst::Movq<$o, AddressTable<Label>>
        where
            W: Write + SectionWrite,
        {
            fn write_inst(&self, w: &mut W) -> io::Result<()> {
                w.movq(self.0, memory(Rip + 0i32))?;
                w.use_relative(-4, self.1 .0, -4, RelocType::PcRelToAddressTable32);
                Ok(())
            }
        }
    };
}

impl_movq_o_label!(Gpr64);
impl_movq_o_label!(Xmm);

// movq AddressTable<Label> O are not provided since address tables are readonly in most cases

/// leaq Gpr64 Label
impl<W> WriteInst<W> for super::inst::Leaq<Gpr64, Label>
where
    W: Write + SectionWrite,
{
    fn write_inst(&self, w: &mut W) -> io::Result<()> {
        w.leaq(self.0, memory(Rip + 0i32))?;
        w.use_relative(-4, self.1, -4, RelocType::PcRel32);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::asm::*;
    use std::io::{self, Write};

    fn write(f: impl FnOnce(&mut Writer) -> io::Result<()>) -> Object {
        let mut w = Writer::new();
        f(&mut w).unwrap();
        w.produce().unwrap()
    }

    #[test]
    fn text() {
        let mut fib_location = Location::new(LocationSection::Bss, 0);
        let mut fib2_location = fib_location.clone();
        let mut fib3_location = fib_location.clone();
        let mut call_fib_location = fib_location.clone();
        let mut call_fib2_location = fib_location.clone();

        let o = write(|w| {
            let fib = w.get_label("fib");
            let fib2 = w.get_label("fib2");
            let fib3 = w.get_label("fib3");
            let l1 = w.issue_label();
            let l2 = w.issue_label();

            fib_location = w.define(fib, true);
            {
                w.cmpl(Edi, 0x1i8)?;
                w.jle(Short(l2))?;
                w.movl(Edx, 0x1)?;
                w.movl(Eax, 0x1)?;
                w.movl(Ecx, 0x0)?;
            }
            w.define(l1, false);
            {
                w.movl(Esi, Eax)?;
                w.addl(Eax, Ecx)?;
                w.addl(Edx, 0x1i8)?;
                w.movl(Ecx, Esi)?;
                w.cmpl(Edi, Edx)?;
                w.jne(Short(l1))?;
                w.retq()?;
            }
            w.define(l2, false);
            {
                w.movl(Eax, 0x1)?;
                w.retq()?;
            }

            fib2_location = w.define(fib2, true);
            {
                w.addl(Edi, 0x1i8)?;
                w.callq(AddressTable(fib))?;
                call_fib_location = w.location();
                w.retq()?;
            }

            fib3_location = w.define(fib3, true);
            {
                w.addl(Edi, 0x1i8)?;
                w.callq(AddressTable(fib2))?;
                call_fib2_location = w.location();
                w.retq()?;
            }

            Ok(())
        });

        assert_asm!(
            o.text,
            r#"
                fib:
                  cmp edi, 0x1
                  jle 2f
                  mov edx, 0x1
                  mov eax, 0x1
                  mov ecx, 0x0
                1:
                  mov esi, eax
                  add eax, ecx
                  add edx, 0x1
                  mov ecx, esi
                  cmp edi, edx
                  jne 1b
                  ret
                2:
                  mov eax, 0x1
                  ret

                fib2:
                  add edi, 0x1
                  call [rip + fib@GOTPCREL]
                  ret

                fib3:
                  add edi, 0x1
                  call [rip + fib2@GOTPCREL]
                  ret
            "#,
        );

        assert_eq!(
            o.symbols,
            [
                Symbol::new("fib".to_string(), Binding::Global(Some(fib_location))),
                Symbol::new("fib2".to_string(), Binding::Global(Some(fib2_location))),
                Symbol::new("fib3".to_string(), Binding::Global(Some(fib3_location))),
            ]
            .to_vec(),
        );
        assert_eq!(
            o.relocs,
            vec![
                Reloc::new(
                    call_fib_location.offset(-4),
                    RelocTarget::Symbol("fib".to_string()),
                    -4,
                    RelocType::PcRelToAddressTable32
                ),
                Reloc::new(
                    call_fib2_location.offset(-4),
                    RelocTarget::Symbol("fib2".to_string()),
                    -4,
                    RelocType::PcRelToAddressTable32
                ),
            ]
        );
    }

    #[test]
    fn data() {
        let o = write(|w| {
            let foo = w.get_label("foo");
            let bar = w.get_label("bar");
            let baz = w.get_label("baz");

            w.data().define(foo, true);
            w.data().write_all(&[0, 1, 2, 3, 4])?;
            w.data().align(2)?;
            w.data().define(bar, true);
            w.data().write_all(&[5, 6, 7, 8])?;
            w.rodata().define(baz, false);
            w.rodata().write_all(&[0; 16])?;
            w.rodata().use_relative(-8, bar, 0, RelocType::Abs64);
            Ok(())
        });

        assert_eq!(
            o.symbols,
            [
                Symbol::new(
                    "bar".to_string(),
                    Binding::Global(Some(Location::new(LocationSection::Data, 6)))
                ),
                Symbol::new(
                    "baz".to_string(),
                    Binding::Local(Location::new(LocationSection::Rodata, 0)),
                ),
                Symbol::new(
                    "foo".to_string(),
                    Binding::Global(Some(Location::new(LocationSection::Data, 0)))
                ),
            ]
            .to_vec(),
        );
        assert_eq!(
            o.relocs,
            vec![Reloc::new(
                Location::new(LocationSection::Rodata, 8),
                RelocTarget::Symbol("bar".to_string()),
                0,
                RelocType::Abs64
            )]
        );
    }

    #[test]
    fn bss() {
        let o = write(|w| {
            let foo = w.get_label("foo");
            let bar = w.get_label("bar");
            let baz = w.get_label("baz");

            w.bss().define(foo, true);
            w.bss().allocate(4)?;

            w.bss().align(2)?;
            w.bss().define(bar, true);
            w.bss().allocate(2)?;

            w.bss().align(8)?;
            w.bss().define(baz, true);
            w.bss().allocate(8)?;
            Ok(())
        });

        assert_eq!(
            o.symbols,
            [
                Symbol::new(
                    "bar".to_string(),
                    Binding::Global(Some(Location::new(LocationSection::Bss, 4)))
                ),
                Symbol::new(
                    "baz".to_string(),
                    Binding::Global(Some(Location::new(LocationSection::Bss, 8))),
                ),
                Symbol::new(
                    "foo".to_string(),
                    Binding::Global(Some(Location::new(LocationSection::Bss, 0)))
                ),
            ]
            .to_vec(),
        );
    }
}
