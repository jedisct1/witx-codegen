use super::*;
use std::io::Write;

impl<W: Write> Generator<W> {
    pub fn define_func(
        &mut self,
        module_name: &str,
        func_witx: &witx::Function,
    ) -> Result<(), Error> {
        let w = &mut self.w;
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

        // A tuple in a result is expanded into additional parameters, transformed to pointers
        if let ASType::Tuple(tuple_members) = ok_type.as_ref() {
            for (i, tuple_member) in tuple_members.iter().enumerate() {
                let ok_type_mut_ptr = ASType::MutPtr(tuple_member.type_.clone());
                params.push((format!("res{}_ptr", i).as_var(), ok_type_mut_ptr))
            }
        } else {
            let ok_type_mut_ptr = ASType::MutPtr(ok_type);
            params.push(("res_ptr".as_var(), ok_type_mut_ptr))
        }

        w.write_line("// @ts-ignore: decorator")?
            .write_line(format!("@external(\"{}\", \"{}\")", module_name, name))?
            .write(format!("export declare function {}(", name.as_fn()))?;
        if !params.is_empty() {
            w.eol()?;
        }
        for (i, param) in params.iter().enumerate() {
            let eol = if i + 1 == params.len() { "" } else { "," };
            w.continuation()?;
            w.write_line(format!("{}: {}{}", param.0.as_var(), param.1, eol))?;
        }

        w.write_line(format!("): {};", result.error_type))?;
        w.eob()?;

        let signature_witx = func_witx.wasm_signature(witx::CallMode::DefinedImport);
        let params_count_witx = signature_witx.params.len() + signature_witx.results.len();
        assert_eq!(params_count_witx, params.len() + 1);

        Ok(())
    }
}
