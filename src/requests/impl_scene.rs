/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use super::*;
use crate::scene::Requests as SceneRequests;

impl SceneRequests for Requests {
    fn update_opperation(&mut self, op: Arc<dyn Operation>) {
        self.operation_update = Some(op);
    }

    fn set_candidate(&mut self, candidates: Vec<Selection>) {
        self.new_candidates = Some(candidates);
    }

    fn set_selection(
        &mut self,
        selection: Vec<Selection>,
        center_of_selection: Option<ensnano_interactor::CenterOfSelection>,
    ) {
        self.new_selection = Some(selection);
        self.new_center_of_selection = Some(center_of_selection);
    }

    fn set_paste_candidate(&mut self, nucl: Option<Nucl>) {
        self.new_paste_candiate = Some(nucl);
    }

    fn attempt_paste(&mut self, nucl: Option<Nucl>) {
        self.keep_proceed.push_back(Action::PasteCandidate(nucl));
        self.keep_proceed.push_back(Action::ApplyPaste);
    }

    fn xover_request(&mut self, source: Nucl, target: Nucl, _design_id: usize) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::GeneralXover {
                source,
                target,
            }))
    }

    fn suspend_op(&mut self) {
        self.suspend_op = Some(());
    }

    fn request_center_selection(&mut self, selection: Selection, app_id: AppId) {
        self.center_selection = Some((selection, app_id));
    }

    fn undo(&mut self) {
        self.undo = Some(());
    }

    fn redo(&mut self) {
        self.redo = Some(());
    }

    fn update_builder_position(&mut self, position: isize) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::MoveBuilders(
                position,
            )))
    }

    fn toggle_widget_basis(&mut self) {
        self.toggle_widget_basis = Some(())
    }

    fn apply_design_operation(&mut self, op: DesignOperation) {
        self.keep_proceed.push_back(Action::DesignOperation(op))
    }
}
