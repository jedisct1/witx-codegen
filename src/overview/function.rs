use super::*;
use std::io::Write;

impl OverviewGenerator {
    pub fn define_func<T: Write>(
        w: &mut PrettyWriter<T>,
        _module_name: &str,
        func_witx: &witx::Function,
    ) -> Result<(), Error> {
        assert_eq!(func_witx.abi, witx::Abi::Preview1);
        let name = func_witx.name.as_str().to_string();
        let params_witx = &func_witx.params;
        let mut params = vec![];
        for param_witx in params_witx {
            let param_name = param_witx.name.as_str();
            let param_type = ASType::from(&param_witx.tref);
            params.push((param_name.to_string(), param_type));
        }

        let results_witx = &func_witx.results;
        assert_eq!(results_witx.len(), 1);
        let result_witx = &results_witx[0];
        let result = ASType::from(&result_witx.tref);
        let result = match result {
            ASType::Result(result) => result,
            _ => unreachable!(),
        };

        let ok_type = result.ok_type.clone();

        let mut results = vec![];
        // A tuple in a result is expanded into additional parameters, transformed to pointers
        if let ASType::Tuple(tuple_members) = ok_type.as_ref().leaf() {
            for (i, tuple_member) in tuple_members.iter().enumerate() {
                let name = format!("result{}_ptr", i);
                results.push((name, tuple_member.type_.clone()));
            }
        } else {
            let name = "result";
            results.push((name.to_string(), ok_type));
        }

        w.write_line(format!(
            "function {}(): {}",
            name.as_fn(),
            result.error_type.as_lang()
        ))?;
        if !params.is_empty() {
            let mut w = w.new_block();
            w.write_line("- Input:")?;
            {
                let mut w = w.new_block();
                for param in &params {
                    w.write_line(format!("- {}: {}", param.0.as_var(), param.1.as_lang()))?;
                }
            }
        }
        if !results.is_empty() {
            let mut w = w.new_block();
            match results[0].1.as_ref() {
                ASType::Void if results.len() == 1 => {
                    w.write_line("- No output")?;
                }
                _ => {
                    w.write_line("- Output:")?;
                    {
                        let mut w = w.new_block();
                        for result in &results {
                            let result_as_ptr = ASType::MutPtr(result.1.clone());
                            w.write_line(format!("- {}", result_as_ptr.as_lang()))?;
                        }
                    }
                }
            }
        }
        w.eob()?;
        Ok(())
    }
}
