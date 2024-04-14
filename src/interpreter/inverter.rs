use crate::parsing::parser::{Block, Stmt};

fn invert_block(block: Block) -> Block {
    let mut inverted_block = vec![];
    for stmt in block {
        // invert each statement in the block to the front
        inverted_block.insert(0, invert_stat(stmt));
    }
    inverted_block
}
pub(crate) fn invert_stat(stat: Stmt) -> Stmt {
    match stat {
        Stmt::Assign(var, value) => Stmt::Unassign(var, value),
        Stmt::If(condition, then_branch, else_branch) => {
            Stmt::If(
                condition,
                invert_block(then_branch),
                invert_block(else_branch),
            )
        }
        Stmt::While(condition, block, acc) => {
            Stmt::WhileR(condition, invert_block(block), acc)
        }
        Stmt::Function(name, args, block) => {
            Stmt::Function(name, args, invert_block(block))
        }
        Stmt::Skip => Stmt::Skip,
        Stmt::Unassign(var, value) => {
            Stmt::Assign(var, value)
        }
        Stmt::WhileR(condition, block, acc) => {
            Stmt::While(condition, invert_block(block), acc)
        }
        Stmt::Return(value) => {
            Stmt::Return(value) // TODO: Add an unreturn?
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::parser::{ArithmeticExpr, BooleanExpr, Stmt};

    #[test]
    fn test_invert_assignment() {
        let stat = Stmt::Assign("x".to_string(), ArithmeticExpr::Num(1));
        let inverted_stat = invert_stat(stat);
        assert_eq!(inverted_stat, Stmt::Unassign("x".to_string(), ArithmeticExpr::Num(1)));
    }

    #[test]
    fn test_invert_if() {
        let stat = Stmt::If(
            BooleanExpr::Bop("==".to_string(), ArithmeticExpr::Num(1), ArithmeticExpr::Num(1)),
            vec![Stmt::Assign("x".to_string(), ArithmeticExpr::Num(1)), Stmt::Assign("y".to_string(), ArithmeticExpr::Num(2))],
            vec![Stmt::Assign("y".to_string(), ArithmeticExpr::Num(2))],
        );
        let inverted_stat = invert_stat(stat);
        assert_eq!(
            inverted_stat,
            Stmt::If(
                BooleanExpr::Bop("==".to_string(), ArithmeticExpr::Num(1), ArithmeticExpr::Num(1)),
                vec![Stmt::Unassign("y".to_string(), ArithmeticExpr::Num(2)), Stmt::Unassign("x".to_string(), ArithmeticExpr::Num(1))],
                vec![Stmt::Unassign("y".to_string(), ArithmeticExpr::Num(2))],
            )
        );
    }

    #[test]
    fn test_invert_while() {
        let stat = Stmt::While(
            BooleanExpr::Bop("==".to_string(), ArithmeticExpr::Num(1), ArithmeticExpr::Num(1)),
            vec![Stmt::Assign("x".to_string(), ArithmeticExpr::Num(1)), Stmt::Assign("y".to_string(), ArithmeticExpr::Num(2)), Stmt::Assign("z".to_string(), ArithmeticExpr::Num(3))],
            BooleanExpr::False,
        );
        let inverted_stat = invert_stat(stat);
        assert_eq!(
            inverted_stat,
            Stmt::While(
                BooleanExpr::Bop("==".to_string(), ArithmeticExpr::Num(1), ArithmeticExpr::Num(1)),
                vec![Stmt::Unassign("z".to_string(), ArithmeticExpr::Num(3)),Stmt::Unassign("y".to_string(), ArithmeticExpr::Num(2)), Stmt::Unassign("x".to_string(), ArithmeticExpr::Num(1))],
                BooleanExpr::False,
            )
        );
    }

    #[test]
    fn test_invert_function() {
        let stat = Stmt::Function(
            "foo".to_string(),
            vec!["x".to_string(), "y".to_string()],
            vec![Stmt::Assign("x".to_string(), ArithmeticExpr::Num(1)), Stmt::Assign("y".to_string(), ArithmeticExpr::Num(2)), Stmt::Return(ArithmeticExpr::Num(3))],
        );
        let inverted_stat = invert_stat(stat);
        assert_eq!(
            inverted_stat,
            Stmt::Function(
                "foo".to_string(),
                vec!["x".to_string(), "y".to_string()],
                vec![Stmt::Return(ArithmeticExpr::Num(3)),Stmt::Unassign("y".to_string(), ArithmeticExpr::Num(2)), Stmt::Unassign("x".to_string(), ArithmeticExpr::Num(1))],
            )
        );
    }

    #[test]
    fn test_invert_skip() {
        let stat = Stmt::Skip;
        let inverted_stat = invert_stat(stat);
        assert_eq!(inverted_stat, Stmt::Skip);
    }

    #[test]
    fn test_invert_unassign() {
        let stat = Stmt::Unassign("x".to_string(), ArithmeticExpr::Num(1));
        let inverted_stat = invert_stat(stat);
        assert_eq!(inverted_stat, Stmt::Assign("x".to_string(), ArithmeticExpr::Num(1)));
    }

    #[test]
    fn test_invert_block() {
        let block = vec![
            Stmt::Assign("x".to_string(), ArithmeticExpr::Num(1)),
            Stmt::Assign("y".to_string(), ArithmeticExpr::Num(2)),
        ];
        let inverted_block = invert_block(block);
        assert_eq!(
            inverted_block,
            vec![
                Stmt::Unassign("y".to_string(), ArithmeticExpr::Num(2)),
                Stmt::Unassign("x".to_string(), ArithmeticExpr::Num(1)),
            ]
        );
    }
}
