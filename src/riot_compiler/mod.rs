use boa_engine::{Context, JsValue, Source};
fn compile_riot_component(riot_code: &str) -> Result<JsValue, Box<dyn std::error::Error>> {
    let mut context = Context::default();
    let compiler = include_str!("riot_compiler.js"); 
    context.eval(Source::from_bytes(compiler))?;
    let result = context.eval(Source::from_bytes(&format!(
        "compileRiot(`{}`)",
        riot_code.replace('`', "\\`")
    )))?;
    Ok(result)
}
fn main() {
    let riot_component = r#"
        <todo-item>
            <h3>{ props.title }</h3>
            <input if="{ !props.done }" type="checkbox" onclick="{ toggle }">
            <span if="{ props.done }">✓ Done</span>
            <script>
                export default {
                    toggle() {
                        this.props.done = !this.props.done
                    }
                }
            </script>
        </todo-item>
    "#;
    match compile_riot_component(riot_component) {
        Ok(compiled) => println!("Compiled: {:?}", compiled),
        Err(e) => eprintln!("Compilation failed: {}", e),
    }
}