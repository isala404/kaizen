use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct TouchDragState {
    long_press_task: Signal<Option<dioxus::core::Task>>,
    touch_active: Signal<bool>,
    touch_start_pos: Signal<(f64, f64)>,
    last_touch_pos: Signal<(f64, f64)>,
}

pub fn use_touch_drag() -> TouchDragState {
    TouchDragState {
        long_press_task: use_signal(|| Option::<dioxus::core::Task>::None),
        touch_active: use_signal(|| false),
        touch_start_pos: use_signal(|| (0.0, 0.0)),
        last_touch_pos: use_signal(|| (0.0, 0.0)),
    }
}

impl TouchDragState {
    pub fn handle_touch_start(
        mut self,
        event: Event<TouchData>,
        item_id: String,
        handler: Option<EventHandler<(String, f64, f64)>>,
    ) {
        if let Some(handler) = handler
            && let Some(touch) = event.touches().first()
        {
            let coordinates = touch.client_coordinates();
            self.touch_start_pos.set((coordinates.x, coordinates.y));
            self.last_touch_pos.set((coordinates.x, coordinates.y));

            let mut touch_active = self.touch_active;
            let task = spawn(async move {
                dioxus_sdk::time::sleep(std::time::Duration::from_millis(300)).await;
                touch_active.set(true);
                handler.call((item_id, coordinates.x, coordinates.y));
            });

            self.long_press_task.set(Some(task));
        }
    }

    pub fn handle_touch_move(
        mut self,
        event: Event<TouchData>,
        handler: Option<EventHandler<(f64, f64)>>,
    ) {
        if let Some(touch) = event.touches().first() {
            let coordinates = touch.client_coordinates();
            self.last_touch_pos.set((coordinates.x, coordinates.y));

            if self.long_press_task.read().is_some() {
                let (start_x, start_y) = *self.touch_start_pos.read();
                let delta_x = coordinates.x - start_x;
                let delta_y = coordinates.y - start_y;
                if (delta_x * delta_x + delta_y * delta_y) > 100.0
                    && let Some(task) = self.long_press_task.take()
                {
                    task.cancel();
                }
                return;
            }

            if *self.touch_active.read() {
                event.prevent_default();
                if let Some(handler) = handler {
                    handler.call((coordinates.x, coordinates.y));
                }
            }
        }
    }

    pub fn handle_touch_end(mut self, handler: Option<EventHandler<(f64, f64)>>) {
        if let Some(task) = self.long_press_task.take() {
            task.cancel();
            return;
        }

        if *self.touch_active.read() {
            self.touch_active.set(false);
            if let Some(handler) = handler {
                handler.call(*self.last_touch_pos.read());
            }
        }
    }
}
