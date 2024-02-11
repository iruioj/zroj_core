use super::ast::*;

impl super::ast::Node {
    pub fn child_nodes(&mut self) -> Option<Vec<&mut Node>> {
        match self {
            Node::Root(Root { children, .. })
            | Node::BlockQuote(BlockQuote { children, .. })
            | Node::List(List { children, .. })
            | Node::FootnoteDefinition(FootnoteDefinition { children, .. })
            | Node::Delete(Delete { children, .. })
            | Node::Emphasis(Emphasis { children, .. })
            | Node::Link(Link { children, .. })
            | Node::LinkReference(LinkReference { children, .. })
            | Node::Strong(Strong { children, .. })
            | Node::Heading(Heading { children, .. })
            | Node::Table(Table { children, .. })
            | Node::TableRow(TableRow { children, .. })
            | Node::TableCell(TableCell { children, .. })
            | Node::ListItem(ListItem { children, .. })
            | Node::Paragraph(Paragraph { children, .. }) => Some(children.iter_mut().collect()),
            Node::TwoColumns(TwoColumns {
                ref mut left,
                ref mut right,
            }) => Some(vec![left, right]),
            _ => None,
        }
    }
}
