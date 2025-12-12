// TODO 管理撤销/重做栈 (Undo/Redo Stack)。

use super::buffer::Buffer;

pub struct EditHistory {
    // 后面我们会用更高效的方式（Chunk）来存，而不是存整个 Buffer 的快照
    // 这里先留空
}

impl EditHistory {
    pub fn new() -> Self {
        Self {}
    }
}
