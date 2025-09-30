/// 递归定义，头元素在栈上，后面元素在堆上，空元素也得分配内存。
///
/// `[A, next] -> (B, next) -> (Empty)`
///
/// c 风格
///
/// `[p] -> (A, next) -> (b, null)`
#[derive(Debug)]
enum Stack<T> {
    Empty,
    Elem {
        val: T,
        next: Box<Stack<T>>,
        len: usize, // 从这个节点到末尾的长度
    },
}

impl<T> Stack<T> {
    /// 创建空链表
    fn new() -> Self {
        Stack::Empty
    }

    /// 在头部添加元素，返回新的链表
    fn push(self, val: T) -> Self {
        let new_len = 1 + self.len();
        Stack::Elem {
            val,
            next: Box::new(self),
            len: new_len,
        }
    }

    /// 获取链表长度（O(1)）
    fn len(&self) -> usize {
        match self {
            Stack::Empty => 0,
            Stack::Elem { len, .. } => *len,
        }
    }

    /// 检查是否为空
    fn is_empty(&self) -> bool {
        matches!(self, Stack::Empty)
    }
    /// 消费式取头
    fn pop(self) -> Option<(T, Stack<T>)> {
        match self {
            Stack::Empty => None,
            Stack::Elem { val, next, .. } => Some((val, *next)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::adlist::Stack;

    #[test]
    pub fn test_adlist() {
        let mut stack = Stack::new()
            .push(3)
            .push(2)
            .push(1);

        // println!("stack = {:?}", stack);
        // println!("stack = {}", stack.len());

        let mut cur = stack;
        while !cur.is_empty() {
            let (head, tail) = cur.pop().expect("should have head");
            println!("pop: {}", head);
            println!("stack: {:?}", tail);
            cur = tail;
        }
    }
}