use std::{io::Write, rc::Rc};

use super::*;

impl WeetGenerator {
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

        let mut i = 0;
        while i < params.len() {
            let param = &params[i];
            let name = &param.0;
            if (i + 1) < params.len() {
                let next_param = &params[i + 1];
                let next_name = &next_param.0;
                let name_with_len = format!("{}_len", name);
                if &name_with_len == next_name {
                    let type_ = Rc::new(ASType::Slice(Rc::new(param.1.clone())));
                    let mut decomposed = vec![ASTypeDecomposed {
                        name: name.clone(),
                        type_,
                    }];
                    params_decomposed.append(&mut decomposed);
                    i += 2;
                    continue;
                }
            }
            let mut decomposed = vec![ASTypeDecomposed {
                name: param.0.clone(),
                type_: Rc::new(param.1.clone()),
            }];
            params_decomposed.append(&mut decomposed);
            i += 1;
        }

        let mut results = vec![];
        // A tuple in a result is expanded into additional parameters, transformed to
        // pointers
        if let ASType::Tuple(tuple_members) = ok_type.as_ref().leaf() {
            for (i, tuple_member) in tuple_members.iter().enumerate() {
                let name = format!("result{}_ptr", i);
                results.push((name, tuple_member.type_.clone()));
            }
        } else {
            let name = "result_ptr";
            results.push((name.to_string(), ok_type));
        }
        let mut results_decomposed = vec![];
        if results.len() < 2 {
            for result in &results {
                let mut decomposed = vec![ASTypeDecomposed {
                    name: result.0.clone(),
                    type_: result.1.clone(),
                }];
                results_decomposed.append(&mut decomposed);
            }
        } else {
            let mut tuple_content: Vec<ASTupleMember> = vec![];
            for result in &results {
                let tuple_member = ASTupleMember {
                    type_: result.1.clone(),
                    padding: 0,
                    offset: 0,
                };
                tuple_content.push(tuple_member);
            }
            let tuple = ASType::Tuple(tuple_content);
            let mut decomposed = vec![ASTypeDecomposed {
                name: "result_tuple".to_string(),
                type_: Rc::new(tuple),
            }];
            results_decomposed.append(&mut decomposed);
        }

        Self::define_func_raw(
            w,
            module_name,
            &name,
            &params_decomposed,
            &results_decomposed,
            &result,
        )?;

        Ok(())
    }

    fn define_func_raw<T: Write>(
        w: &mut PrettyWriter<T>,
        _module_name: &str,
        name: &str,
        params_decomposed: &[ASTypeDecomposed],
        results_decomposed: &[ASTypeDecomposed],
        result: &ASResult,
    ) -> Result<(), Error> {
        w.indent()?.write(format!("{}: func(", name.as_fn()))?;
        if !params_decomposed.is_empty() || !results_decomposed.is_empty() {
            w.eol()?;
        }
        for param in params_decomposed.iter() {
            w.write_line_continued(format!(
                "{}: {},",
                param.name.as_var(),
                param.type_.as_lang(),
            ))?;
        }
        w.indent()?.write(") -> result<")?;
        for param in results_decomposed.iter() {
            w.write(format!("{}", param.type_.as_lang(),))?;
        }
        w.write(format!(", {}>;", result.error_type.as_lang()))?
            .eol()?;
        w.eob()?;
        Ok(())
    }
}
