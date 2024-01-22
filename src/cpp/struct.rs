use super::*;
use std::io::Write;

impl CppGenerator {
    pub fn define_as_struct<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        members: &[ASStructMember],
    ) -> Result<(), Error> {
        w
            // .write_line("#[repr(C, packed)]")?
            .write_line(format!(
                "struct __attribute__((packed)) {} {{",
                name.as_type()
            ))?;
        {
            let mut w = w.new_block();
            for member in members {
                let member_type = member.type_.as_ref();
                w.write_line(format!(
                    "{} {};",
                    member_type.as_lang(),
                    member.name.as_var()
                ))?;

                let pad_len = member.padding;
                for i in 0..(pad_len & 1) {
                    w.write_line(format!("uint8_t __pad8_{};", i))?;
                }
                for i in 0..(pad_len & 3) / 2 {
                    w.write_line(format!("uint16_t __pad16_{};", i))?;
                }
                for i in 0..(pad_len & 7) / 4 {
                    w.write_line(format!("uint32_t __pad32_{};", i))?;
                }
                for i in 0..pad_len / 8 {
                    w.write_line(format!("uint64_t __pad64_{};", i))?;
                }
            }
        }
        w.write_line("};")?.eob()?;

        for member in members {
            w.write_line(format!(
                "static_assert(offsetof({}, {}) == {}, \"Error layout\");",
                name.as_type(),
                member.name.as_var(),
                member.offset
            ))?;
        }

        Ok(())
    }
}
