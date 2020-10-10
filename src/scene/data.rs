//! This modules handles internal informations about the scene, such as the selected objects etc..
//! It also communicates with the desgings to get the position of the objects to draw on the scene.

use super::{View, ViewUpdate};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use ultraviolet::{Rotor3, Vec3};

use crate::design::{Design, ObjectType, Referential};
use crate::mediator::Selection;
use crate::utils::instance::Instance;

type ViewPtr = Rc<RefCell<View>>;

/// A module that handles the instantiation of designs as 3D geometric objects
mod design3d;
use design3d::Design3D;

pub struct Data {
    view: ViewPtr,
    /// A `Design3D` is associated to each design.
    designs: Vec<Design3D>,
    /// The set of selected elements represented by `(design identifier, group identifier)`
    selected: Vec<(u32, u32)>,
    /// The set of candidates elements represented by `(design identifier, group identifier)`
    candidates: Vec<(u32, u32)>,
    /// The kind of selection being perfomed on the scene.
    pub selection_mode: SelectionMode,
    /// The kind of action being performed on the scene
    pub action_mode: ActionMode,
    /// A position determined by the current selection. If only one nucleotide is selected, it's
    /// the position of the nucleotide.
    selected_position: Option<Vec3>,
    selection_update: bool,
    candidate_update: bool,
    instance_update: bool,
    matrices_update: bool,
    widget_basis: Option<WidgetBasis>,
}

impl Data {
    pub fn new(view: ViewPtr) -> Self {
        Self {
            view,
            designs: Vec::new(),
            selected: Vec::new(),
            candidates: Vec::new(),
            selection_mode: SelectionMode::default(),
            action_mode: Default::default(),
            selected_position: None,
            selection_update: false,
            candidate_update: false,
            instance_update: false,
            matrices_update: false,
            widget_basis: None,
        }
    }

    /// Add a new design to be drawn
    pub fn add_design(&mut self, design: Arc<Mutex<Design>>) {
        self.designs.push(Design3D::new(design));
        self.notify_instance_update();
        self.notify_matrices_update();
    }

    /// Remove all designs to be drawn
    pub fn clear_designs(&mut self) {
        self.designs = Vec::new();
        self.selected = Vec::new();
        self.candidates = Vec::new();
        self.reset_selection();
        self.reset_candidate();
        self.notify_instance_update();
        self.notify_matrices_update();
    }

    /// Forwards all needed update to the view
    pub fn update_view(&mut self) {
        if self.instance_update {
            self.update_instances();
            self.instance_update = false;
        }

        if self.selection_update {
            self.update_selection();
            self.selection_update = false;
        }
        if self.candidate_update {
            self.update_candidate();
            self.candidate_update = false;
        }

        if self.matrices_update {
            self.update_matrices();
            self.matrices_update = false;
        }
    }

    /// Return the sets of selected designs
    pub fn get_selected_designs(&self) -> HashSet<u32> {
        self.selected.iter().map(|x| x.0).collect()
    }

    /// Convert `self.selection` into a set of elements according to `self.selection_mode`
    fn expand_selection(&self, object_type: ObjectType) -> HashSet<(u32, u32)> {
        let mut ret = HashSet::new();
        for (d_id, elt_id) in &self.selected {
            let group_id = self.get_group_identifier(*d_id, *elt_id);
            let group = self.get_group_member(*d_id, group_id);
            for elt in group.iter() {
                if self.designs[*d_id as usize]
                    .get_element_type(*elt)
                    .unwrap()
                    .same_type(object_type)
                {
                    ret.insert((*d_id, *elt));
                }
            }
        }
        ret
    }

    /// Convert `self.candidates` into a set of elements according to `self.selection_mode`
    fn expand_candidate(&self, object_type: ObjectType) -> HashSet<(u32, u32)> {
        let mut ret = HashSet::new();
        for (d_id, elt_id) in &self.candidates {
            let group_id = self.get_group_identifier(*d_id, *elt_id);
            let group = self.get_group_member(*d_id, group_id);
            for elt in group.iter() {
                if self.designs[*d_id as usize]
                    .get_element_type(*elt)
                    .unwrap()
                    .same_type(object_type)
                {
                    ret.insert((*d_id, *elt));
                }
            }
        }
        ret
    }

    /// Return the instances of selected spheres
    pub fn get_selected_spheres(&self) -> Rc<Vec<Instance>> {
        let mut ret = Vec::with_capacity(self.selected.len());
        for (d_id, id) in self.expand_selection(ObjectType::Nucleotide(0)).iter() {
            ret.push(self.designs[*d_id as usize].make_instance(*id))
        }
        Rc::new(ret)
    }

    /// Return the instances of selected tubes
    pub fn get_selected_tubes(&self) -> Rc<Vec<Instance>> {
        let mut ret = Vec::with_capacity(self.selected.len());
        for (d_id, id) in self.expand_selection(ObjectType::Bound(0, 0)).iter() {
            ret.push(self.designs[*d_id as usize].make_instance(*id))
        }
        Rc::new(ret)
    }

    /// Return the instances of candidate spheres
    pub fn get_candidate_spheres(&self) -> Rc<Vec<Instance>> {
        let mut ret = Vec::with_capacity(self.selected.len());
        for (d_id, id) in self.expand_candidate(ObjectType::Nucleotide(0)).iter() {
            ret.push(self.designs[*d_id as usize].make_instance(*id))
        }
        Rc::new(ret)
    }

    /// Return the instances of candidate tubes
    pub fn get_candidate_tubes(&self) -> Rc<Vec<Instance>> {
        let mut ret = Vec::with_capacity(self.selected.len());
        for (d_id, id) in self.expand_candidate(ObjectType::Bound(0, 0)).iter() {
            ret.push(self.designs[*d_id as usize].make_instance(*id))
        }
        Rc::new(ret)
    }

    /// Return the identifier of the first selected group
    pub fn get_selected_group(&self) -> u32 {
        self.get_group_identifier(self.selected[0].0, self.selected[0].1)
    }

    /// Return the group to which an element belongs. The group depends on self.selection_mode.
    fn get_group_identifier(&self, design_id: u32, element_id: u32) -> u32 {
        match self.selection_mode {
            SelectionMode::Nucleotide => element_id,
            SelectionMode::Design => design_id,
            SelectionMode::Strand => self.designs[design_id as usize].get_strand(element_id),
            SelectionMode::Helix => self.designs[design_id as usize].get_helix(element_id),
        }
    }

    /// Return the set of elements in a given group
    fn get_group_member(&self, design_id: u32, group_id: u32) -> HashSet<u32> {
        match self.selection_mode {
            SelectionMode::Nucleotide => vec![group_id].into_iter().collect(),
            SelectionMode::Design => self.designs[design_id as usize].get_all_elements(),
            SelectionMode::Strand => self.designs[design_id as usize].get_strand_elements(group_id),
            SelectionMode::Helix => self.designs[design_id as usize].get_helix_elements(group_id),
        }
    }

    /// Return the postion of a given element, either in the world pov or in the model pov
    pub fn get_element_position(
        &self,
        design_id: u32,
        element_id: u32,
        referential: Referential,
    ) -> Vec3 {
        self.designs[design_id as usize]
            .get_element_position(element_id, referential)
            .unwrap()
    }

    pub fn get_selected_position(&self) -> Option<Vec3> {
        self.selected_position
    }

    /// Update the selection by selecting the group to which a given nucleotide belongs. Return the
    /// selected group
    pub fn set_selection(&mut self, design_id: u32, element_id: u32) -> Selection {
        let future_selection = vec![(design_id, element_id)];
        if self.selected == future_selection {
            self.toggle_widget_basis()
        } else {
            self.widget_basis = Some(WidgetBasis::World);
            self.selection_update = true;
        }
        self.selected = future_selection;
        self.selected_position = {
            self.selected.get(0).map(|(design_id, element_id)| {
                self.get_element_position(*design_id, *element_id, Referential::World)
            })
        };
        let group_id = self.get_group_identifier(design_id, element_id);
        match self.selection_mode {
            SelectionMode::Design => Selection::Design(design_id),
            SelectionMode::Strand => Selection::Strand(design_id, group_id),
            SelectionMode::Nucleotide => Selection::Nucleotide(design_id, group_id),
            SelectionMode::Helix => Selection::Helix(design_id, group_id),
        }
    }

    /// This function must be called when the current movement ends.
    pub fn end_movement(&mut self) {
        self.selected_position = {
            self.selected.get(0).map(|(design_id, element_id)| {
                self.get_element_position(*design_id, *element_id, Referential::World)
            })
        };
    }

    /// Clear self.selected
    pub fn reset_selection(&mut self) {
        self.selection_update |= !self.selected.is_empty();
        self.selected_position = None;
        self.selected = Vec::new();
    }

    /// Notify the view that the selected elements have been modified
    fn update_selection(&mut self) {
        self.view
            .borrow_mut()
            .update(ViewUpdate::SelectedTubes(self.get_selected_tubes()));
        self.view
            .borrow_mut()
            .update(ViewUpdate::SelectedSpheres(self.get_selected_spheres()));
        let (sphere, vec) = self.get_phantom_instances();
        self.view
            .borrow_mut()
            .update(ViewUpdate::PhantomInstances(sphere, vec));
    }

    /// Return the sets of elements of the phantom helix
    pub fn get_phantom_instances(&self) -> (Rc<Vec<Instance>>, Rc<Vec<Instance>>) {
        if self.selected.is_empty() {
            return (Rc::new(Vec::new()), Rc::new(Vec::new()));
        }
        match self.selection_mode {
            SelectionMode::Helix => {
                let mut selected_helices = HashSet::new();
                for (d_id, elt_id) in &self.selected {
                    let group_id = self.get_group_identifier(*d_id, *elt_id);
                    selected_helices.insert(group_id);
                }
                self.designs[self.selected[0].0 as usize]
                    .make_phantom_helix_instances(&selected_helices)
            }
            _ => (Rc::new(Vec::new()), Rc::new(Vec::new())),
        }
    }

    /// Set the set of candidates to a given nucleotide
    pub fn set_candidate(&mut self, design_id: u32, element_id: u32) {
        let future_candidate = vec![(design_id, element_id)];
        self.candidate_update |= self.candidates == future_candidate;
        self.candidates = future_candidate;
    }

    /// Clear the set of candidates to a given nucleotide
    pub fn reset_candidate(&mut self) {
        self.candidate_update |= !self.candidates.is_empty();
        self.candidates = Vec::new();
    }

    /// Notify the view that the instances of candidates have changed
    fn update_candidate(&mut self) {
        self.view
            .borrow_mut()
            .update(ViewUpdate::CandidateTubes(self.get_candidate_tubes()));
        self.view
            .borrow_mut()
            .update(ViewUpdate::CandidateSpheres(self.get_candidate_spheres()));
    }

    /// This function must be called when the designs have been modified
    pub fn notify_instance_update(&mut self) {
        self.instance_update = true;
    }

    /// Notify the view that the set of instances have been modified.
    fn update_instances(&mut self) {
        let mut spheres = Vec::with_capacity(self.get_number_spheres());
        let mut tubes = Vec::with_capacity(self.get_number_tubes());

        for design in self.designs.iter() {
            for sphere in design.get_spheres().iter() {
                spheres.push(*sphere);
            }
            for tube in design.get_tubes().iter() {
                tubes.push(*tube);
            }
        }
        self.view
            .borrow_mut()
            .update(ViewUpdate::Tubes(Rc::new(tubes)));
        self.view
            .borrow_mut()
            .update(ViewUpdate::Spheres(Rc::new(spheres)));
    }

    /// This fuction must be called when the model matrices have been modfied
    pub fn notify_matrices_update(&mut self) {
        self.matrices_update = true;
    }

    /// Notify the view of an update of the model matrices
    fn update_matrices(&mut self) {
        let mut matrices = Vec::new();
        for design in self.designs.iter() {
            matrices.push(design.get_model_matrix());
        }
        self.view
            .borrow_mut()
            .update(ViewUpdate::ModelMatrices(matrices));
    }

    /// Return a position and rotation of the camera that fits the first design
    pub fn get_fitting_camera(&self, ratio: f32, fovy: f32) -> Option<(Vec3, Rotor3)> {
        let design = self.designs.get(0)?;
        Some(design.get_fitting_camera(ratio, fovy))
    }

    /// Return the point in the middle of the selected design
    pub fn get_middle_point(&self, design_id: u32) -> Vec3 {
        self.designs[design_id as usize].middle_point()
    }

    fn get_number_spheres(&self) -> usize {
        self.designs.iter().map(|d| d.get_spheres().len()).sum()
    }

    fn get_number_tubes(&self) -> usize {
        self.designs.iter().map(|d| d.get_tubes().len()).sum()
    }

    pub fn toggle_selection_mode(&mut self) {
        self.selection_mode = match self.selection_mode {
            SelectionMode::Nucleotide => SelectionMode::Design,
            SelectionMode::Design => SelectionMode::Strand,
            SelectionMode::Strand => SelectionMode::Helix,
            SelectionMode::Helix => SelectionMode::Nucleotide,
        }
    }

    pub fn change_selection_mode(&mut self, selection_mode: SelectionMode) {
        self.selection_mode = selection_mode;
    }

    pub fn get_action_mode(&self) -> ActionMode {
        self.action_mode
    }

    pub fn change_action_mode(&mut self, action_mode: ActionMode) {
        self.action_mode = action_mode;
    }

    pub fn toggle_widget_basis(&mut self) {
        self.widget_basis.as_mut().map(|w| w.toggle());
    }

    pub fn get_widget_basis(&self) -> Rotor3 {
        match self.widget_basis.as_ref().expect("widget basis") {
            WidgetBasis::World => Rotor3::identity(),
            WidgetBasis::Object => self.get_selected_basis().unwrap(),
        }
    }

    fn get_selected_basis(&self) -> Option<Rotor3> {
        let (d_id, e_id) = self.selected[0];
        match self.selection_mode {
            SelectionMode::Nucleotide | SelectionMode::Design | SelectionMode::Strand => {
                Some(self.designs[d_id as usize].get_basis())
            }
            SelectionMode::Helix => {
                let h_id = self.get_selected_group();
                self.designs[d_id as usize].get_helix_basis(h_id)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Nucleotide,
    Design,
    Strand,
    Helix,
}

impl Default for SelectionMode {
    fn default() -> Self {
        SelectionMode::Nucleotide
    }
}

impl std::fmt::Display for SelectionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SelectionMode::Design => "Design",
                SelectionMode::Nucleotide => "Nucleotide",
                SelectionMode::Strand => "Strand",
                SelectionMode::Helix => "Helix",
            }
        )
    }
}

impl SelectionMode {
    pub const ALL: [SelectionMode; 4] = [
        SelectionMode::Nucleotide,
        SelectionMode::Design,
        SelectionMode::Strand,
        SelectionMode::Helix,
    ];
}

/// Describe the action currently done by the user when they click left
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionMode {
    /// User is moving the camera
    Normal,
    /// User can translate objects and move the camera
    Translate,
    /// User can rotate objects and move the camera
    Rotate,
    /// User can elongate/shorten strands
    Build,
}

impl Default for ActionMode {
    fn default() -> Self {
        ActionMode::Normal
    }
}

impl std::fmt::Display for ActionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ActionMode::Normal => "Normal",
                ActionMode::Translate => "Translate",
                ActionMode::Rotate => "Rotate",
                ActionMode::Build => "Build",
            }
        )
    }
}

impl ActionMode {
    pub const ALL: [ActionMode; 4] = [
        ActionMode::Normal,
        ActionMode::Translate,
        ActionMode::Rotate,
        ActionMode::Build,
    ];
}

#[derive(Clone, Copy)]
enum WidgetBasis {
    World,
    Object,
}

impl WidgetBasis {
    pub fn toggle(&mut self) {
        match self {
            WidgetBasis::World => *self = WidgetBasis::Object,
            WidgetBasis::Object => *self = WidgetBasis::World,
        }
    }
}
