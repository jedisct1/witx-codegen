use super::*;
use std::io::Write;

impl AssemblyScriptGenerator {
    pub fn define_func<T: Write>(
        w: &mut PrettyWriter<T>,
        module_name: &str,
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

        let docs = &func_witx.docs;
        if !docs.is_empty() {
            Self::write_docs(w, docs)?;
        }

        let mut params_decomposed = vec![];

        for param in &params {
            let mut decomposed = param.1.decompose(&param.0, false);
            params_decomposed.append(&mut decomposed);
        }

        let mut results = vec![];
        // A tuple in a result is expanded into additional parameters, transformed to pointers
        if let ASType::Tuple(tuple_members) = ok_type.as_ref().leaf() {
            for (i, tuple_member) in tuple_members.iter().enumerate() {
                let name = format!("result{}_ptr", i);
                results.push((name, tuple_member.type_.clone()));
            }
        } else {
            let name = "result_ptr";
            results.push((name.to_string(), ok_type));
        }
        for result in &results {
            let mut decomposed = result.1.decompose(&result.0, true);
            params_decomposed.append(&mut decomposed);
        }

        w.write_line("// @ts-ignore: decorator")?
            .write_line(format!("@external(\"{}\", \"{}\")", module_name, name))?
            .indent()?
            .write(format!("export declare function {}(", name.as_fn()))?;

        if !params_decomposed.is_empty() || !results.is_empty() {
            w.eol()?;
        }
        for (i, param) in params_decomposed.iter().enumerate() {
            let eol = if i + 1 == params_decomposed.len() {
                ""
            } else {
                ","
            };
            w.write_line_continued(format!(
                "{}: {}{}",
                param.name.as_var(),
                param.type_.as_lang(),
                eol
            ))?;
        }

        w.write_line(format!("): {};", result.error_type.as_lang()))?;
        w.eob()?;

        let signature_witx = func_witx.wasm_signature(witx::CallMode::DefinedImport);
        let params_count_witx = signature_witx.params.len() + signature_witx.results.len();
        assert_eq!(params_count_witx, params_decomposed.len() + 1);

        Ok(())
    }
}
