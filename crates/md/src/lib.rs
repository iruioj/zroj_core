pub mod ast;
pub use ast::Node;
/// Turn markdown into a syntax tree.
///
/// ## Errors
///
/// `to_mdast()` never errors with normal markdown because markdown does not
/// have syntax errors, so feel free to `unwrap()`.
/// However, MDX does have syntax errors.
/// When MDX is turned on, there are several errors that can occur with how
/// JSX, expressions, or ESM are written.
///
/// ## Examples
///
/// ```
/// use md::{parse_ast};
/// # fn main() -> Result<(), String> {
///
/// let tree = parse_ast("# Hey, *you*!")?;
///
/// println!("{:?}", tree);
/// // => Root { children: [Heading { children: [Text { value: "Hey, " }, Emphasis { children: [Text { value: "you" }] }, Text { value: "!" }], depth: 1 }] }
/// # Ok(())
/// # }
/// ```
pub fn parse_ast(content: &str) -> Result<ast::Node, String> {
    let mut opt = markdown::ParseOptions::gfm();
    opt.math_text_single_dollar = true;
    opt.constructs.math_text = true;
    opt.constructs.math_flow = true;
    markdown::to_mdast(content, &opt).map(|a| a.into())
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let tree = parse_ast("hello $a + b$ baba\n\n$$\na^2 + b^2 = c^2\n$$").unwrap();

        println!("{:?}", tree);
    }
}
