use gpui::*;

#[derive(Clone, Debug)]
pub struct ScrollOffset {
    pub item_index: usize,
    #[allow(dead_code)]
    pub offset_in_item: Pixels,
}

pub struct ScrollState {
    item_count: usize,
    visible_range: std::ops::Range<usize>,
    scroll_offset: ScrollOffset,
    _item_height: Pixels,
}

impl ScrollState {
    pub fn new(item_height: Pixels) -> Self {
        Self {
            item_count: 0,
            visible_range: 0..0,
            scroll_offset: ScrollOffset {
                item_index: 0,
                offset_in_item: px(0.0),
            },
            _item_height: item_height,
        }
    }

    pub fn update_item_count(&mut self, count: usize) {
        self.item_count = count;
        if self.scroll_offset.item_index >= count && count > 0 {
            self.scroll_offset.item_index = count - 1;
        }
    }

    pub fn scroll_to_reveal_item(&mut self, index: usize, visible_items: usize) {
        if index >= self.item_count {
            return;
        }

        let half_visible = visible_items / 2;

        // Calculer la nouvelle position de scroll pour centrer l'élément sélectionné
        let new_scroll_index = if index < half_visible {
            0
        } else if index + half_visible >= self.item_count {
            self.item_count.saturating_sub(visible_items)
        } else {
            index - half_visible
        };

        self.scroll_offset = ScrollOffset {
            item_index: new_scroll_index,
            offset_in_item: px(0.0),
        };

        self.update_visible_range(visible_items);
    }

    pub fn update_visible_range(&mut self, visible_items: usize) {
        let start = self.scroll_offset.item_index;
        let end = (start + visible_items).min(self.item_count);
        self.visible_range = start..end;
    }

    pub fn visible_range(&self) -> std::ops::Range<usize> {
        self.visible_range.clone()
    }
}
