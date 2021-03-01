use rhai::{Dynamic, Engine, Map, Scope, AST};
pub struct ScriptEngine {
    engine: Engine,
    ast: AST,
}

#[derive(Clone)]
pub struct EvalContext {
    pub headers: Map,
}

impl ScriptEngine {
    pub fn new(script: &str) -> ScriptEngine {
        let engine = Engine::new();
        let ast = engine
            .compile(&format!(
                r#"
        fn main(headers) {{
          {}
        }}
        let result = main(headers);

        "#,
                script
            ))
            .expect("Parse error");
        ScriptEngine { engine, ast }
    }

    pub fn run(&self, context: &EvalContext) -> Option<i64> {
        let mut scope = Scope::new();
        scope.set_value("headers", context.headers.clone());
        let result = self
            .engine
            .eval_ast_with_scope::<Dynamic>(&mut scope, &self.ast);
        if result.is_err() {
            result.unwrap();
            return None;
        }

        let result = scope.get_value::<Dynamic>("result").unwrap().as_int();
        match result {
            Ok(x) => Some(x),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rhai::ImmutableString;

    use super::*;

    #[test]
    fn reads_and_finds_a_header() {
        let mut headers: HashMap<ImmutableString, Dynamic> = HashMap::new();
        headers.insert("My-Header".into(), "".into());
        let ctx = EvalContext { headers };
        let engine = ScriptEngine::new(
            r#"
      for header in headers.keys() {
        if header == "My-Header" {
          return 200;
        }
      }
    "#,
        );
        assert_eq!(engine.run(&ctx), Some(200));
    }

    #[test]
    fn no_return_statement() {
        let headers: HashMap<ImmutableString, Dynamic> = HashMap::new();
        let ctx = EvalContext { headers };
        let engine = ScriptEngine::new(
            r#"
  "#,
        );
        assert_eq!(engine.run(&ctx), None);
    }

    #[test]
    fn call_native_function() {
        let mut headers: HashMap<ImmutableString, Dynamic> = HashMap::new();
        headers.insert("My-Header1".into(), "".into());
        headers.insert("My-Header2".into(), "".into());
        headers.insert("My-Header3".into(), "".into());
        headers.insert("My-Header4".into(), "".into());
        headers.insert("My-Header5".into(), "".into());
        let ctx = EvalContext { headers };
        let engine = ScriptEngine::new(
            r#"
  let cnt = 0;
  for header in headers.keys() {
    cnt += 1;
  }
  return cnt;

"#,
        );
        assert_eq!(engine.run(&ctx), Some(5));
    }
}
