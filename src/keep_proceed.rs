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

//! This modules defines action that are performed immediatly after handling the requests.

/// An action to be performed immediately after processing a Request.
#[derive(Debug, Clone)]
pub enum KeepProceed {
    DefaultScaffold,
    CustomScaffold,
    OptimizeShift(usize),
    Stapples(usize),
    Quit,
    LoadDesign,
    LoadDesignAfterSave,
    SaveBeforeQuit,
    SaveBeforeOpen,
    SaveBeforeNew,
    NewDesign,
    NewDesignAfterSave,
    Other,
}
