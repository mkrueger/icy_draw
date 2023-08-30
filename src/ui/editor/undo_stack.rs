use icy_engine::AttributedChar;

use super::{Buffer, Layer, Position, Size};

pub trait UndoOperation: Send {
    fn undo(&self, buffer: &mut Buffer);
    fn redo(&self, buffer: &mut Buffer);
}

pub struct UndoSetChar {
    pub pos: Position,
    pub layer: usize,
    pub old: AttributedChar,
    pub new: AttributedChar,
}

impl UndoOperation for UndoSetChar {
    fn undo(&self, buffer: &mut Buffer) {
        buffer.layers[self.layer].set_char(self.pos, self.old);
    }

    fn redo(&self, buffer: &mut Buffer) {
        buffer.layers[self.layer].set_char(self.pos, self.new);
    }
}

pub struct AtomicUndo {
    pub stack: Vec<Box<dyn UndoOperation>>,
}

impl UndoOperation for AtomicUndo {
    fn undo(&self, buffer: &mut Buffer) {
        for op in &self.stack {
            op.undo(buffer);
        }
    }

    fn redo(&self, buffer: &mut Buffer) {
        for op in self.stack.iter().rev() {
            op.redo(buffer);
        }
    }
}

pub struct UndoSwapChar {
    pub layer: usize,
    pub pos1: Position,
    pub pos2: Position,
}
impl UndoOperation for UndoSwapChar {
    fn undo(&self, buffer: &mut Buffer) {
        buffer.layers[self.layer].swap_char(self.pos1, self.pos2);
    }

    fn redo(&self, buffer: &mut Buffer) {
        buffer.layers[self.layer].swap_char(self.pos1, self.pos2);
    }
}

pub struct UndoReplaceLayers {
    pub old_layer: Vec<Layer>,
    pub new_layer: Vec<Layer>,
    pub old_size: Size,
    pub new_size: Size,
}

impl UndoOperation for UndoReplaceLayers {
    fn undo(&self, _buffer: &mut Buffer) {
        /* TODO
        buffer.layers = self.old_layer.clone();
        buffer.set_buffer_width(self.old_size.width);
        buffer.set_buffer_height(self.old_size.height);*/
    }

    fn redo(&self, _buffer: &mut Buffer) {
        /* TODO
        buffer.layers = self.new_layer.clone();
        buffer.set_buffer_width(self.new_size.width);
        buffer.set_buffer_height(self.new_size.height);*/
    }
}

pub struct ClearLayerOperation {
    pub layer_num: usize,
    pub lines: Vec<super::Line>,
}

impl UndoOperation for ClearLayerOperation {
    fn undo(&self, buffer: &mut Buffer) {
        buffer.layers[self.layer_num].lines.clear();
        buffer.layers[self.layer_num]
            .lines
            .extend(self.lines.clone());
    }

    fn redo(&self, buffer: &mut Buffer) {
        buffer.layers[self.layer_num].lines.clear();
    }
}
