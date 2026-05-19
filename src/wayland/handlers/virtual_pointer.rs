// SPDX-License-Identifier: GPL-3.0-only

use crate::{state::State, utils::prelude::*};
use smithay::{
    input::{Seat, pointer::MotionEvent},
    utils::{Logical, Point, SERIAL_COUNTER},
    wayland::virtual_pointer::VirtualPointerHandler,
};

impl VirtualPointerHandler for State {
    fn virtual_pointer_get_default_seat(&self) -> Option<Seat<Self>> {
        Some(self.common.shell.read().seats.last_active().clone())
    }

    fn virtual_pointer_motion(
        &mut self,
        seat: Option<&Seat<Self>>,
        time: u32,
        delta: Point<f64, Logical>,
    ) {
        let seat = match seat {
            Some(s) => s.clone(),
            None => self.common.shell.read().seats.last_active().clone(),
        };

        self.common.idle_notifier_state.notify_activity(&seat);

        let ptr = match seat.get_pointer() {
            Some(p) => p,
            None => return,
        };

        let (output, position, under) = {
            let shell = self.common.shell.read();
            let current_output = seat.active_output();
            let mut position = ptr.current_location().as_global();
            position += delta.as_global();

            let output = shell
                .outputs()
                .find(|o| o.geometry().to_f64().contains(position))
                .cloned()
                .unwrap_or(current_output);

            let geo = output.geometry();
            position.x = position
                .x
                .clamp(geo.loc.x as f64, (geo.loc.x + geo.size.w - 1) as f64);
            position.y = position
                .y
                .clamp(geo.loc.y as f64, (geo.loc.y + geo.size.h - 1) as f64);

            let under =
                State::surface_under(position, &output, &shell).map(|(t, p)| (t, p.as_logical()));

            (output, position, under)
        };

        let _ = output;
        ptr.motion(
            self,
            under,
            &MotionEvent {
                location: position.as_logical(),
                serial: SERIAL_COUNTER.next_serial(),
                time,
            },
        );
        ptr.frame(self);
    }

    fn virtual_pointer_motion_absolute(
        &mut self,
        seat: Option<&Seat<Self>>,
        time: u32,
        x: u32,
        y: u32,
        x_extent: u32,
        y_extent: u32,
    ) {
        if x_extent == 0 || y_extent == 0 {
            return;
        }

        let seat = match seat {
            Some(s) => s.clone(),
            None => self.common.shell.read().seats.last_active().clone(),
        };

        self.common.idle_notifier_state.notify_activity(&seat);

        let ptr = match seat.get_pointer() {
            Some(p) => p,
            None => return,
        };

        let (position, under) = {
            let shell = self.common.shell.read();
            let geo = seat.active_output().geometry();

            let position = Point::from((
                geo.loc.x as f64 + (x as f64 / x_extent as f64) * geo.size.w as f64,
                geo.loc.y as f64 + (y as f64 / y_extent as f64) * geo.size.h as f64,
            ))
            .as_global();

            let output = seat.active_output();
            let under =
                State::surface_under(position, &output, &shell).map(|(t, p)| (t, p.as_logical()));

            (position, under)
        };

        ptr.motion(
            self,
            under,
            &MotionEvent {
                location: position.as_logical(),
                serial: SERIAL_COUNTER.next_serial(),
                time,
            },
        );
        ptr.frame(self);
    }
}
