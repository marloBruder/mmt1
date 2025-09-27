use crate::model::ParseTreeNode;

pub struct ParseTreeNodeIterator<'a> {
    nodes_to_check: Vec<&'a ParseTreeNode>,
}

impl<'a> ParseTreeNodeIterator<'a> {
    pub fn new(parse_tree_node: &'a ParseTreeNode) -> ParseTreeNodeIterator<'a> {
        ParseTreeNodeIterator {
            nodes_to_check: vec![parse_tree_node],
        }
    }
}

impl<'a> Iterator for ParseTreeNodeIterator<'a> {
    type Item = &'a ParseTreeNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.nodes_to_check.pop()?;

        if let ParseTreeNode::Node {
            rule_i: _,
            sub_nodes,
        } = node
        {
            self.nodes_to_check.extend(sub_nodes.iter());
        }

        Some(node)
    }
}
