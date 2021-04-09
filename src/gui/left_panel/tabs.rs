use super::*;
use iced::scrollable;

pub(super) struct EditionTab {
    selection_mode_state: SelectionModeState,
    action_mode_state: ActionModeState,
    scroll: iced::scrollable::State,
    helix_roll_factory: RequestFactory<HelixRoll>,
    color_picker: ColorPicker,
    sequence_input: SequenceInput,
}

impl EditionTab {
    pub(super) fn new() -> Self {
        Self {
            selection_mode_state: Default::default(),
            action_mode_state: Default::default(),
            scroll: Default::default(),
            helix_roll_factory: RequestFactory::new(FactoryId::HelixRoll, HelixRoll {}),
            color_picker: ColorPicker::new(),
            sequence_input: SequenceInput::new(),
        }
    }

    pub(super) fn view<'a>(
        &'a mut self,
        action_mode: ActionMode,
        selection_mode: SelectionMode,
        ui_size: UiSize,
        width: u16,
    ) -> Element<'a, Message> {
        let mut ret = Column::new().spacing(5);
        ret = ret.push(
            Text::new("Edition")
                .horizontal_alignment(iced::HorizontalAlignment::Center)
                .size(ui_size.main_text() * 2),
        );
        let selection_modes = [
            SelectionMode::Nucleotide,
            SelectionMode::Strand,
            SelectionMode::Helix,
        ];

        let mut selection_buttons: Vec<Button<'a, Message>> = self
            .selection_mode_state
            .get_states()
            .into_iter()
            .rev()
            .filter(|(m, _)| selection_modes.contains(m))
            .map(|(mode, state)| selection_mode_btn(state, mode, selection_mode, ui_size.button()))
            .collect();

        ret = ret.push(Text::new("Selection Mode"));
        while selection_buttons.len() > 0 {
            let mut row = Row::new();
            row = row.push(selection_buttons.pop().unwrap()).spacing(5);
            let mut space = ui_size.button() + 5;
            while space + ui_size.button() < width && selection_buttons.len() > 0 {
                row = row.push(selection_buttons.pop().unwrap()).spacing(5);
                space += ui_size.button() + 5;
            }
            ret = ret.push(row)
        }

        let action_modes = [
            ActionMode::Normal,
            ActionMode::Translate,
            ActionMode::Rotate,
        ];

        let mut action_buttons: Vec<Button<'a, Message>> = self
            .action_mode_state
            .get_states(0, 0)
            .into_iter()
            .filter(|(m, _)| action_modes.contains(m))
            .map(|(mode, state)| action_mode_btn(state, mode, action_mode, ui_size.button()))
            .collect();

        ret = ret.push(Text::new("Action Mode"));
        while action_buttons.len() > 0 {
            let mut row = Row::new();
            row = row.push(action_buttons.remove(0)).spacing(5);
            let mut space = ui_size.button() + 5;
            while space + ui_size.button() < width && action_buttons.len() > 0 {
                row = row.push(action_buttons.remove(0)).spacing(5);
                space += ui_size.button() + 5;
            }
            ret = ret.push(row)
        }

        if selection_mode == SelectionMode::Helix {
            for view in self.helix_roll_factory.view().into_iter() {
                ret = ret.push(view);
            }
        }

        let color_square = self.color_picker.color_square();
        if selection_mode == SelectionMode::Strand {
            ret = ret
                .push(self.color_picker.view())
                .push(
                    Row::new()
                        .push(color_square)
                        .push(iced::Space::new(Length::FillPortion(4), Length::Shrink)),
                )
                .push(self.sequence_input.view());
        }

        Scrollable::new(&mut self.scroll).push(ret).into()
    }

    pub(super) fn update_roll(&mut self, roll: f32) {
        self.helix_roll_factory.update_roll(roll);
    }

    pub(super) fn update_roll_request(
        &mut self,
        value_id: ValueId,
        value: f32,
        request: &mut Option<f32>,
    ) {
        self.helix_roll_factory
            .update_request(value_id, value, request);
    }
}

pub(super) struct GridTab {
    selection_mode_state: SelectionModeState,
    action_mode_state: ActionModeState,
    scroll: iced::scrollable::State,
    helix_pos: isize,
    helix_length: usize,
    pos_str: String,
    length_str: String,
    builder_input: [text_input::State; 2],
    building_hyperboloid: bool,
    finalize_hyperboloid_btn: button::State,
    make_grid_btn: button::State,
    hyperboloid_factory: RequestFactory<Hyperboloid_>,
    start_hyperboloid_btn: button::State,
}

impl GridTab {
    pub fn new() -> Self {
        Self {
            selection_mode_state: Default::default(),
            action_mode_state: Default::default(),
            scroll: Default::default(),
            helix_pos: 0,
            helix_length: 0,
            pos_str: "0".to_owned(),
            length_str: "0".to_owned(),
            builder_input: Default::default(),
            make_grid_btn: Default::default(),
            hyperboloid_factory: RequestFactory::new(FactoryId::Hyperboloid, Hyperboloid_ {}),
            finalize_hyperboloid_btn: Default::default(),
            building_hyperboloid: false,
            start_hyperboloid_btn: Default::default(),
        }
    }

    pub(super) fn view<'a>(
        &'a mut self,
        action_mode: ActionMode,
        selection_mode: SelectionMode,
        ui_size: UiSize,
        width: u16,
    ) -> Element<'a, Message> {
        let mut ret = Column::new().spacing(5);
        ret = ret.push(
            Text::new("Grids")
                .horizontal_alignment(iced::HorizontalAlignment::Center)
                .size(ui_size.main_text() * 2),
        );
        let selection_modes = [
            SelectionMode::Nucleotide,
            SelectionMode::Strand,
            SelectionMode::Helix,
        ];

        let mut selection_buttons: Vec<Button<'a, Message>> = self
            .selection_mode_state
            .get_states()
            .into_iter()
            .rev()
            .filter(|(m, _)| selection_modes.contains(m))
            .map(|(mode, state)| selection_mode_btn(state, mode, selection_mode, ui_size.button()))
            .collect();

        ret = ret.push(Text::new("Selection Mode"));
        while selection_buttons.len() > 0 {
            let mut row = Row::new();
            row = row.push(selection_buttons.pop().unwrap()).spacing(5);
            let mut space = ui_size.button() + 5;
            while space + ui_size.button() < width && selection_buttons.len() > 0 {
                row = row.push(selection_buttons.pop().unwrap()).spacing(5);
                space += ui_size.button() + 5;
            }
            ret = ret.push(row)
        }

        let action_modes = [
            ActionMode::Normal,
            ActionMode::Translate,
            ActionMode::Rotate,
            ActionMode::BuildHelix {
                position: self.helix_pos,
                length: self.helix_length,
            },
        ];

        let mut action_buttons: Vec<Button<'a, Message>> = self
            .action_mode_state
            .get_states(self.helix_length, self.helix_pos)
            .into_iter()
            .filter(|(m, _)| action_modes.contains(m))
            .map(|(mode, state)| action_mode_btn(state, mode, action_mode, ui_size.button()))
            .collect();

        ret = ret.push(Text::new("Action Mode"));
        while action_buttons.len() > 0 {
            let mut row = Row::new();
            row = row.push(action_buttons.remove(0)).spacing(5);
            let mut space = ui_size.button() + 5;
            while space + ui_size.button() < width && action_buttons.len() > 0 {
                row = row.push(action_buttons.remove(0)).spacing(5);
                space += ui_size.button() + 5;
            }
            ret = ret.push(row)
        }

        let mut inputs = self.builder_input.iter_mut();
        let position_input = TextInput::new(
            inputs.next().unwrap(),
            "Position",
            &self.pos_str,
            Message::PositionHelicesChanged,
        )
        .style(BadValue(self.pos_str == self.helix_pos.to_string()));

        let length_input = TextInput::new(
            inputs.next().unwrap(),
            "Length",
            &self.length_str,
            Message::LengthHelicesChanged,
        )
        .style(BadValue(self.length_str == self.helix_length.to_string()));

        if let ActionMode::BuildHelix { .. } = action_mode {
            let row = Row::new()
                .push(
                    Column::new()
                        .push(Text::new("Position strand").color(Color::WHITE))
                        .push(position_input)
                        .width(Length::Units(width / 2)),
                )
                .push(
                    Column::new()
                        .push(Text::new("Length strands").color(Color::WHITE))
                        .push(length_input),
                );
            ret = ret.push(row);
        }

        ret = ret.push(iced::Space::with_height(Length::Units(5)));

        let make_grid_btn = text_btn(&mut self.make_grid_btn, "Make Grid", ui_size.clone())
            .on_press(Message::NewGrid);

        ret = ret.push(make_grid_btn);

        ret = ret.push(iced::Space::with_height(Length::Units(5)));

        let start_hyperboloid_btn = text_btn(
            &mut self.start_hyperboloid_btn,
            "Start Hyperboloid",
            ui_size.clone(),
        )
        .on_press(Message::NewHyperboloid);

        ret = ret.push(start_hyperboloid_btn);
        if self.building_hyperboloid {
            for view in self.hyperboloid_factory.view().into_iter() {
                ret = ret.push(view);
            }
            ret = ret.push(
                text_btn(
                    &mut self.finalize_hyperboloid_btn,
                    "Finish",
                    ui_size.clone(),
                )
                .on_press(Message::FinalizeHyperboloid),
            );
        }

        Scrollable::new(&mut self.scroll).push(ret).into()
    }

    pub(super) fn update_pos_str(&mut self, position_str: String) -> ActionMode {
        if let Ok(position) = position_str.parse::<isize>() {
            self.helix_pos = position;
        }
        self.pos_str = position_str;
        ActionMode::BuildHelix {
            position: self.helix_pos,
            length: self.helix_length,
        }
    }

    pub(super) fn update_length_str(&mut self, length_str: String) -> ActionMode {
        if let Ok(length) = length_str.parse::<usize>() {
            self.helix_length = length
        }
        self.length_str = length_str;
        ActionMode::BuildHelix {
            position: self.helix_pos,
            length: self.helix_length,
        }
    }

    pub fn has_keyboard_priority(&self) -> bool {
        self.builder_input.iter().any(|s| s.is_focused())
    }

    pub fn new_hyperboloid(&mut self, requests: &mut Option<HyperboloidRequest>) {
        self.hyperboloid_factory = RequestFactory::new(FactoryId::Hyperboloid, Hyperboloid_ {});
        self.hyperboloid_factory.make_request(requests);
        self.building_hyperboloid = true;
    }

    pub fn finalize_hyperboloid(&mut self) {
        self.building_hyperboloid = false;
    }

    pub fn update_hyperboloid_request(
        &mut self,
        value_id: ValueId,
        value: f32,
        request: &mut Option<HyperboloidRequest>,
    ) {
        self.hyperboloid_factory
            .update_request(value_id, value, request);
    }
}

fn selection_mode_btn<'a>(
    state: &'a mut button::State,
    mode: SelectionMode,
    fixed_mode: SelectionMode,
    button_size: u16,
) -> Button<'a, Message> {
    let icon_path = if fixed_mode == mode {
        mode.icon_on()
    } else {
        mode.icon_off()
    };

    Button::new(state, Image::new(icon_path))
        .on_press(Message::SelectionModeChanged(mode))
        .style(ButtonStyle(fixed_mode == mode))
        .width(Length::Units(button_size))
}

fn action_mode_btn<'a>(
    state: &'a mut button::State,
    mode: ActionMode,
    fixed_mode: ActionMode,
    button_size: u16,
) -> Button<'a, Message> {
    let icon_path = if fixed_mode == mode {
        mode.icon_on()
    } else {
        mode.icon_off()
    };

    Button::new(state, Image::new(icon_path))
        .on_press(Message::ActionModeChanged(mode))
        .style(ButtonStyle(fixed_mode == mode))
        .width(Length::Units(button_size))
}

pub(super) struct CameraTab {
    camera_target_buttons: [button::State; 6],
    camera_rotation_buttons: [button::State; 4],
    scroll_sensitivity_factory: RequestFactory<ScrollSentivity>,
    xz: isize,
    yz: isize,
    fog: FogParameters,
    scroll: scrollable::State,
}

impl CameraTab {
    pub fn new() -> Self {
        Self {
            camera_target_buttons: Default::default(),
            camera_rotation_buttons: Default::default(),
            scroll_sensitivity_factory: RequestFactory::new(FactoryId::Scroll, ScrollSentivity {}),
            fog: Default::default(),
            xz: 0,
            yz: 0,
            scroll: Default::default(),
        }
    }

    pub fn view<'a>(&'a mut self, ui_size: UiSize, width: u16) -> Element<'a, Message> {
        let mut ret = Column::new();
        ret = ret.push(
            Text::new("Camera")
                .horizontal_alignment(iced::HorizontalAlignment::Center)
                .size(ui_size.main_text() * 2),
        );
        for view in self.scroll_sensitivity_factory.view().into_iter() {
            ret = ret.push(view);
        }
        let mut target_buttons: Vec<_> = self
            .camera_target_buttons
            .iter_mut()
            .enumerate()
            .map(|(i, s)| {
                Button::new(s, Text::new(target_text(i)).size(10))
                    .on_press(target_message(i))
                    .width(Length::Units(ui_size.button()))
            })
            .collect();
        ret = ret.push(Text::new("Camera Target"));
        while target_buttons.len() > 0 {
            let mut row = Row::new();
            row = row.push(target_buttons.remove(0)).spacing(5);
            let mut space = ui_size.button() + 5;
            while space + ui_size.button() < width && target_buttons.len() > 0 {
                row = row.push(target_buttons.remove(0)).spacing(5);
                space += ui_size.button() + 5;
            }
            ret = ret.push(row)
        }

        let xz = self.xz;
        let yz = self.yz;

        let mut rotate_buttons: Vec<_> = self
            .camera_rotation_buttons
            .iter_mut()
            .enumerate()
            .map(|(i, s)| {
                Button::new(s, rotation_text(i, ui_size.clone()))
                    .on_press(rotation_message(i, xz, yz))
                    .width(Length::Units(ui_size.button()))
            })
            .collect();

        ret = ret.push(Text::new("Rotate Camera"));
        while rotate_buttons.len() > 0 {
            let mut row = Row::new();
            row = row.push(rotate_buttons.remove(0)).spacing(5);
            let mut space = ui_size.button() + 5;
            while space + ui_size.button() < width && rotate_buttons.len() > 0 {
                row = row.push(rotate_buttons.remove(0)).spacing(5);
                space += ui_size.button() + 5;
            }
            ret = ret.spacing(5).push(row)
        }
        ret = ret.push(self.fog.view(&ui_size));

        Scrollable::new(&mut self.scroll).push(ret).into()
    }

    pub(super) fn reset_angles(&mut self) {
        self.xz = 0;
        self.yz = 0;
    }

    pub(super) fn set_angles(&mut self, xz: isize, yz: isize) {
        self.xz = xz;
        self.yz = yz;
    }

    pub(super) fn fog_visible(&mut self, visible: bool) {
        self.fog.visible = visible
    }

    pub(super) fn fog_length(&mut self, length: f32) {
        self.fog.length = length
    }

    pub(super) fn fog_radius(&mut self, radius: f32) {
        self.fog.radius = radius
    }

    pub(super) fn fog_camera(&mut self, from_camera: bool) {
        self.fog.from_camera = from_camera;
    }

    pub(super) fn get_fog_request(&self) -> Fog {
        self.fog.request()
    }

    pub(super) fn notify_new_design(&mut self) {
        self.fog = Default::default();
    }

    pub(super) fn update_scroll_request(
        &mut self,
        value_id: ValueId,
        value: f32,
        request: &mut Option<f32>,
    ) {
        self.scroll_sensitivity_factory
            .update_request(value_id, value, request);
    }
}

struct FogParameters {
    visible: bool,
    from_camera: bool,
    radius: f32,
    radius_slider: slider::State,
    length: f32,
    length_slider: slider::State,
}

impl FogParameters {
    fn view(&mut self, ui_size: &UiSize) -> Column<Message> {
        let mut column = Column::new()
            .push(Text::new("Fog"))
            .push(
                Checkbox::new(self.visible, "Visible", Message::FogVisibility)
                    .size(ui_size.checkbox())
                    .spacing(CHECKBOXSPACING),
            )
            .push(
                Checkbox::new(self.from_camera, "From Camera", Message::FogCamera)
                    .size(ui_size.checkbox())
                    .spacing(CHECKBOXSPACING),
            );

        if self.visible {
            column = column
                .push(Text::new("Radius"))
                .push(Slider::new(
                    &mut self.radius_slider,
                    0f32..=100f32,
                    self.radius,
                    Message::FogRadius,
                ))
                .push(Text::new("Length"))
                .push(Slider::new(
                    &mut self.length_slider,
                    0f32..=100f32,
                    self.length,
                    Message::FogLength,
                ));
        }
        column
    }

    fn request(&self) -> Fog {
        Fog {
            radius: self.radius,
            active: self.visible,
            length: self.length,
            from_camera: self.from_camera,
            alt_fog_center: None,
        }
    }
}

impl Default for FogParameters {
    fn default() -> Self {
        Self {
            visible: false,
            length: 10.,
            radius: 10.,
            length_slider: Default::default(),
            radius_slider: Default::default(),
            from_camera: false,
        }
    }
}

pub(super) struct SimulationTab {
    rigid_body_factory: RequestFactory<RigidBodyFactory>,
    rigid_grid_button: GoStop,
    rigid_helices_button: GoStop,
    scroll: scrollable::State,
    physical_simulation: PhysicalSimulation,
}

impl SimulationTab {
    pub(super) fn new() -> Self {
        Self {
            rigid_body_factory: RequestFactory::new(
                FactoryId::RigidBody,
                RigidBodyFactory {
                    volume_exclusion: false,
                },
            ),
            rigid_helices_button: GoStop::new(
                String::from("Rigid Helices"),
                Message::RigidHelicesSimulation,
            ),
            rigid_grid_button: GoStop::new(
                String::from("Rigid Grids"),
                Message::RigidGridSimulation,
            ),
            scroll: Default::default(),
            physical_simulation: Default::default(),
        }
    }

    pub(super) fn view<'a>(&'a mut self, ui_size: UiSize) -> Element<'a, Message> {
        let mut ret = Column::new();
        ret = ret.push(Text::new("Simulation (Beta)").size(2 * ui_size.main_text()));
        ret = ret.push(self.physical_simulation.view(&ui_size));
        ret = ret
            .push(self.rigid_grid_button.view())
            .push(self.rigid_helices_button.view());

        let volume_exclusion = self.rigid_body_factory.requestable.volume_exclusion;
        for view in self.rigid_body_factory.view().into_iter() {
            ret = ret.push(view);
        }
        ret = ret.push(
            Checkbox::new(
                volume_exclusion,
                "Volume exclusion",
                Message::VolumeExclusion,
            )
            .spacing(CHECKBOXSPACING)
            .size(ui_size.checkbox()),
        );

        Scrollable::new(&mut self.scroll).push(ret).into()
    }

    pub(super) fn notify_grid_running(&mut self, running: bool) {
        self.rigid_grid_button.running = running;
    }

    pub(super) fn notify_helices_running(&mut self, running: bool) {
        self.rigid_helices_button.running = running;
    }

    pub(super) fn set_volume_exclusion(&mut self, volume_exclusion: bool) {
        self.rigid_body_factory.requestable.volume_exclusion = volume_exclusion;
    }

    pub(super) fn make_rigid_body_request(
        &mut self,
        request: &mut Option<RigidBodyParametersRequest>,
    ) {
        self.rigid_body_factory.make_request(request)
    }

    pub(super) fn update_request(
        &mut self,
        value_id: ValueId,
        value: f32,
        request: &mut Option<RigidBodyParametersRequest>,
    ) {
        self.rigid_body_factory
            .update_request(value_id, value, request)
    }

    pub(super) fn notify_new_design(&mut self) {
        self.physical_simulation.running = false;
        self.rigid_grid_button.running = false;
        self.rigid_helices_button.running = false;
    }

    pub(super) fn notify_sim_request(&mut self) {
        self.physical_simulation.running ^= true;
    }

    pub(super) fn set_roll(&mut self, roll: bool) {
        self.physical_simulation.roll = roll;
    }

    pub(super) fn set_springs(&mut self, springs: bool) {
        self.physical_simulation.springs = springs;
    }

    pub(super) fn get_physical_simulation_request(&self) -> SimulationRequest {
        self.physical_simulation.request()
    }
}

struct GoStop {
    go_stop_button: button::State,
    pub running: bool,
    pub name: String,
    on_press: Box<dyn Fn(bool) -> Message>,
}

impl GoStop {
    fn new<F>(name: String, on_press: F) -> Self
    where
        F: 'static + Fn(bool) -> Message,
    {
        Self {
            go_stop_button: Default::default(),
            running: false,
            name,
            on_press: Box::new(on_press),
        }
    }

    fn view(&mut self) -> Row<Message> {
        let left_column = Column::new().push(Text::new(self.name.to_string()));
        let button_str = if self.running { "Stop" } else { "Go" };
        let right_column = Column::new().push(
            Button::new(&mut self.go_stop_button, Text::new(button_str))
                .on_press((self.on_press)(!self.running))
                .style(ButtonColor::red_green(self.running)),
        );
        Row::new().push(left_column).push(right_column)
    }
}

#[derive(Default)]
struct PhysicalSimulation {
    go_stop_button: button::State,
    pub running: bool,
    pub roll: bool,
    pub springs: bool,
}

impl PhysicalSimulation {
    fn view<'a, 'b>(&'a mut self, ui_size: &'b UiSize) -> Row<'a, Message> {
        let left_column = Column::new()
            .push(
                Checkbox::new(self.roll, "Roll", Message::SimRoll)
                    .size(ui_size.checkbox())
                    .spacing(CHECKBOXSPACING),
            )
            .push(
                Checkbox::new(self.springs, "Spring", Message::SimSprings)
                    .size(ui_size.checkbox())
                    .spacing(CHECKBOXSPACING),
            );
        let button_str = if self.running { "Stop" } else { "Go" };
        let right_column = Column::new().push(
            Button::new(&mut self.go_stop_button, Text::new(button_str))
                .on_press(Message::SimRequest),
        );
        Row::new().push(left_column).push(right_column)
    }

    fn request(&self) -> SimulationRequest {
        SimulationRequest {
            roll: self.roll,
            springs: self.springs,
        }
    }
}

pub struct ParametersTab {
    size_pick_list: pick_list::State<UiSize>,
    scroll: scrollable::State,
}

impl ParametersTab {
    pub(super) fn new() -> Self {
        Self {
            size_pick_list: Default::default(),
            scroll: Default::default(),
        }
    }

    pub(super) fn view<'a>(&'a mut self, ui_size: UiSize) -> Element<'a, Message> {
        let mut ret = Column::new();
        ret = ret.push(Text::new("Parameters").size(2 * ui_size.main_text()));
        ret = ret.push(PickList::new(
            &mut self.size_pick_list,
            &super::super::ALL_UI_SIZE[..],
            Some(ui_size.clone()),
            Message::UiSizePicked,
        ));

        Scrollable::new(&mut self.scroll).push(ret).into()
    }
}
