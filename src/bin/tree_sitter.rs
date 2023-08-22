
fn main() -> Result<()> {

    let source_code = r##"
<div hx-boost=true hx-get="/foo" hx-trigger="click delay:200ms" id="content">"##;

    match get_position(root_node, source_code, 1, 45) {
        Some(position) => {
            println!("current: {:?}", position);
        }
        None => todo!(),
    }

    return Ok(());
}
