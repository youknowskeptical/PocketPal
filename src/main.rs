#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod chat;

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
#[cfg(target_os = "windows")]
use std::{
    io::Write,
    os::windows::process::CommandExt,
    process::{Command, Stdio},
};
#[cfg(target_os = "windows")]
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
};

const INK: Color32 = Color32::from_rgb(50, 42, 52);
const CREAM: Color32 = Color32::from_rgb(255, 247, 225);
const ROSE: Color32 = Color32::from_rgb(238, 129, 128);
const MELON: Color32 = Color32::from_rgb(255, 179, 126);
const LEMON: Color32 = Color32::from_rgb(250, 213, 122);
const MINT: Color32 = Color32::from_rgb(156, 204, 169);
const SKY: Color32 = Color32::from_rgb(157, 194, 216);
const APP_BG: Color32 = Color32::from_rgb(8, 8, 8);
const APP_PANEL: Color32 = Color32::from_rgb(14, 14, 14);
const APP_SURFACE: Color32 = Color32::from_rgb(22, 22, 22);
const APP_SURFACE_HI: Color32 = Color32::from_rgb(35, 35, 35);
const APP_TEXT: Color32 = Color32::from_rgb(245, 245, 245);
const APP_MUTED: Color32 = Color32::from_rgb(158, 158, 158);
const APP_BORDER: Color32 = Color32::from_rgb(54, 54, 54);
const APP_ACCENT: Color32 = Color32::from_rgb(220, 220, 220);
const APP_LAVENDER: Color32 = Color32::from_rgb(185, 185, 185);
const APP_FEED: Color32 = Color32::from_rgb(135, 135, 135);
const APP_NAP: Color32 = Color32::from_rgb(105, 105, 105);
const APP_PLAY: Color32 = Color32::from_rgb(170, 170, 170);
const FLOAT_FEED: Color32 = Color32::from_rgb(115, 115, 115);
const FLOAT_NAP: Color32 = Color32::from_rgb(75, 75, 75);
const FLOAT_PLAY: Color32 = Color32::from_rgb(180, 180, 180);
const FLOAT_CHAT: Color32 = Color32::from_rgb(145, 145, 145);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum PetKind {
    Cat,
    Fox,
    Bunny,
    Frog,
}

impl PetKind {
    const ALL: [PetKind; 4] = [PetKind::Cat, PetKind::Fox, PetKind::Bunny, PetKind::Frog];

    fn name(self) -> &'static str {
        match self {
            Self::Cat => "Mochi",
            Self::Fox => "Pip",
            Self::Bunny => "Clover",
            Self::Frog => "Bean",
        }
    }

    fn species(self) -> &'static str {
        match self {
            Self::Cat => "CAT",
            Self::Fox => "FOX",
            Self::Bunny => "BUNNY",
            Self::Frog => "FROG",
        }
    }

    fn unlock_cost(self) -> u32 {
        match self {
            Self::Cat | Self::Frog => 0,
            Self::Fox => 12,
            Self::Bunny => 15,
        }
    }

    fn colors(self) -> (Color32, Color32, Color32) {
        match self {
            Self::Cat => (
                Color32::from_rgb(204, 139, 102),
                Color32::from_rgb(247, 214, 172),
                Color32::from_rgb(133, 86, 73),
            ),
            Self::Fox => (
                Color32::from_rgb(238, 132, 83),
                Color32::from_rgb(255, 224, 177),
                Color32::from_rgb(152, 81, 64),
            ),
            Self::Bunny => (
                Color32::from_rgb(220, 184, 181),
                Color32::from_rgb(255, 234, 224),
                Color32::from_rgb(154, 104, 119),
            ),
            Self::Frog => (
                Color32::from_rgb(123, 177, 115),
                Color32::from_rgb(216, 233, 167),
                Color32::from_rgb(69, 119, 89),
            ),
        }
    }

    fn sprite(self) -> [&'static str; 16] {
        match self {
            Self::Cat => [
                "....DD....DD....",
                "...DFF....FFD...",
                "...DFFF..FFFD...",
                "..DFFFFFFFFFFD..",
                "..FFFFFFFFFFFF..",
                ".FFFFFFFFFFFFFF.",
                ".FFFFFFFFFFFFFF.",
                ".FFEEFFCCFFEEFF.",
                ".FFEEFFCCFFEEFF.",
                ".FFFFFFFFFFFFFF.",
                "..FFFFFPPFFFFF..",
                "...FFFFFFFFFF...",
                "....FFFFFFFF....",
                "...FF.FFFF.FF...",
                "..FFF.FFFF.FFFF.",
                "..DDD......DDD..",
            ],
            Self::Fox => [
                "....DD....DD....",
                "...DFF....FFD...",
                "...DFF....FFD...",
                "...DFFFFFFFFD...",
                "..DFFFFFFFFFFD..",
                ".FFFFFFFFFFFFFF.",
                ".FFFFFFFFFFFFFF.",
                ".FFEEFFCCFFEEFF.",
                ".FFEEFFCCFFEEFF.",
                ".FFFFFFFFFFFFFF.",
                "..FFF..PP..FFF..",
                "...FFFFFFFFFF...",
                "....FFFFFFFF....",
                "...FF.FFFF.FF...",
                "..FFF.FFFF.FFFF.",
                "..DDD......DDD..",
            ],
            Self::Bunny => [
                "...DD......DD...",
                "...DFF....FFD...",
                "...DFF....FFD...",
                "...DFF....FFD...",
                "...DFF....FFD...",
                "..DFFFFFFFFFFD..",
                ".FFFFFFFFFFFFFF.",
                ".FFEEFFCCFFEEFF.",
                ".FFEEFFCCFFEEFF.",
                ".FFFFFFFFFFFFFF.",
                "..FFFFFPPFFFFF..",
                "...FFFFFFFFFF...",
                "....FFFFFFFF....",
                "...FF.FFFF.FF...",
                "..FFF.FFFF.FFFF.",
                "..DDD......DDD..",
            ],
            Self::Frog => [
                "....DD....DD....",
                "...DFF....FFD...",
                "...DFFFFFFFFD...",
                "..DFFFFFFFFFFD..",
                "..FFFFFFFFFFFF..",
                ".FFFFFFFFFFFFFF.",
                ".FFEEFFCCFFEEFF.",
                ".FFEEFFCCFFEEFF.",
                ".FFFFFFFFFFFFFF.",
                ".FFFFFFFFFFFFFF.",
                "..FFFFFPPFFFFF..",
                "...FFFFFFFFFF...",
                "....FFFFFFFF....",
                "...FF.FFFF.FF...",
                "..FFF.FFFF.FFFF.",
                "..DDD......DDD..",
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum Decoration {
    Plant,
    Rug,
    Lamp,
    Picture,
    Cushion,
}

impl Decoration {
    const ALL: [Decoration; 5] = [
        Decoration::Plant,
        Decoration::Rug,
        Decoration::Lamp,
        Decoration::Picture,
        Decoration::Cushion,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Plant => "Window plant",
            Self::Rug => "Sunshine rug",
            Self::Lamp => "Mushroom lamp",
            Self::Picture => "Tiny landscape",
            Self::Cushion => "Star cushion",
        }
    }

    fn cost(self) -> u32 {
        match self {
            Self::Plant => 6,
            Self::Rug => 8,
            Self::Lamp => 10,
            Self::Picture => 12,
            Self::Cushion => 7,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Plant => "A little leafy friend",
            Self::Rug => "For extra cozy naps",
            Self::Lamp => "Soft evening glow",
            Self::Picture => "A view of somewhere nice",
            Self::Cushion => "A star-shaped squish",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct PetNeeds {
    fullness: u8,
    joy: u8,
    energy: u8,
}

impl Default for PetNeeds {
    fn default() -> Self {
        Self {
            fullness: 72,
            joy: 70,
            energy: 82,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SaveData {
    coins: u32,
    selected_pet: PetKind,
    owned_pets: BTreeSet<PetKind>,
    needs: BTreeMap<PetKind, PetNeeds>,
    unlocked_decor: BTreeSet<Decoration>,
    placed_decor: BTreeSet<Decoration>,
    play_count: u32,
    #[serde(default = "default_floating_on_top")]
    floating_on_top: bool,
    #[serde(default)]
    chat_provider: chat::Provider,
    #[serde(default)]
    run_at_startup: bool,
}

fn default_floating_on_top() -> bool {
    true
}

impl Default for SaveData {
    fn default() -> Self {
        let owned_pets = BTreeSet::from([PetKind::Cat, PetKind::Frog]);
        let needs = owned_pets
            .iter()
            .copied()
            .map(|pet| (pet, PetNeeds::default()))
            .collect();
        Self {
            coins: 28,
            selected_pet: PetKind::Cat,
            owned_pets,
            needs,
            unlocked_decor: BTreeSet::new(),
            placed_decor: BTreeSet::new(),
            play_count: 0,
            floating_on_top: true,
            chat_provider: chat::Provider::default(),
            run_at_startup: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Buddy,
    Collection,
    Decor,
    Settings,
    Chat,
}

struct StarGame {
    score: u8,
    time_left: f32,
    target: (f32, f32),
    move_timer: f32,
    target_step: u32,
}

impl StarGame {
    fn new() -> Self {
        Self {
            score: 0,
            time_left: 15.0,
            target: (0.50, 0.50),
            move_timer: 1.1,
            target_step: 0,
        }
    }

    fn move_target(&mut self) {
        self.target_step = self.target_step.wrapping_add(1);
        let step = self.target_step as f32;
        self.target.0 = 0.12 + ((step * 1.71 + 0.19).sin() * 0.5 + 0.5) * 0.76;
        self.target.1 = 0.12 + ((step * 2.37 + 0.83).cos() * 0.5 + 0.5) * 0.76;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PetMood {
    Happy,
    Sleepy,
    Hungry,
    Sad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CareTaskKind {
    Snack,
    Nap,
}

struct CareTask {
    kind: CareTaskKind,
    remaining: f32,
    duration: f32,
}

impl CareTask {
    fn new(kind: CareTaskKind) -> Self {
        let duration = match kind {
            CareTaskKind::Snack => 5.0,
            CareTaskKind::Nap => 7.0,
        };
        Self {
            kind,
            remaining: duration,
            duration,
        }
    }

    fn progress(&self) -> f32 {
        1.0 - self.remaining / self.duration
    }
}

#[derive(Clone)]
struct FloatingOverlayState {
    pet: PetKind,
    needs: PetNeeds,
    care_task: Option<(CareTaskKind, f32)>,
    celebrating: bool,
    celebration_time: f32,
    voice_line: Option<String>,
    game: Option<(u8, f32, (f32, f32))>,
    game_move_timer: f32,
    game_target_step: u32,
    actions_open: bool,
    dirty: bool,
    reward_pending: bool,
    chat: Arc<Mutex<FloatingChatState>>,
    exit_requested: bool,
    restore_requested: bool,
    last_update: Option<f64>,
}

struct FloatingChatState {
    open: bool,
    provider: chat::Provider,
    checked_provider: Option<chat::Provider>,
    saved_key: bool,
    key_input: String,
    messages: Vec<chat::Message>,
    input: String,
    pending: bool,
    reply_receiver: Option<mpsc::Receiver<Result<String, String>>>,
    error: Option<String>,
    voice_enabled: bool,
}

impl FloatingChatState {
    fn new(provider: chat::Provider) -> Self {
        Self {
            open: false,
            provider,
            checked_provider: None,
            saved_key: false,
            key_input: String::new(),
            messages: Vec::new(),
            input: String::new(),
            pending: false,
            reply_receiver: None,
            error: None,
            voice_enabled: cfg!(target_os = "windows"),
        }
    }
}

struct PocketPal {
    save: SaveData,
    tab: Tab,
    game: Option<StarGame>,
    notice: String,
    notice_time: f32,
    floating: bool,
    main_hidden_for_float: bool,
    floating_actions_open: bool,
    care_task: Option<CareTask>,
    celebration_time: f32,
    voice_line: Option<String>,
    speech_bubble_time: f32,
    hunger_voice_triggered: bool,
    voice_line_index: usize,
    floating_overlay: Arc<Mutex<FloatingOverlayState>>,
    chat: ChatSession,
    #[cfg(target_os = "windows")]
    tray: Option<TrayIntegration>,
    #[cfg(target_os = "windows")]
    tray_hidden: bool,
    #[cfg(target_os = "windows")]
    tray_restore_pending: bool,
}

struct ChatSession {
    messages: Vec<chat::Message>,
    input: String,
    key_input: String,
    checked_provider: Option<chat::Provider>,
    saved_key: bool,
    pending: bool,
    reply_receiver: Option<mpsc::Receiver<Result<String, String>>>,
    error: Option<String>,
}

#[cfg(target_os = "windows")]
struct TrayIntegration {
    _icon: TrayIcon,
}

impl PocketPal {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        style_app(&cc.egui_ctx);
        let save = match load_save() {
            Ok(Some(save)) => save,
            Ok(None) => SaveData::default(),
            Err(error) => {
                eprintln!("Pocket Pal could not load its save: {error}");
                SaveData::default()
            }
        };
        let selected_pet = save.selected_pet;
        let floating_chat = Arc::new(Mutex::new(FloatingChatState::new(save.chat_provider)));
        let floating_overlay = Arc::new(Mutex::new(FloatingOverlayState {
            pet: selected_pet,
            needs: save.needs.get(&selected_pet).copied().unwrap_or_default(),
            care_task: None,
            celebrating: false,
            celebration_time: 0.0,
            voice_line: None,
            game: None,
            game_move_timer: 1.1,
            game_target_step: 0,
            actions_open: false,
            dirty: false,
            reward_pending: false,
            chat: floating_chat,
            exit_requested: false,
            restore_requested: false,
            last_update: None,
        }));
        #[cfg(target_os = "windows")]
        let tray = match create_tray(cc.egui_ctx.clone(), Arc::clone(&floating_overlay)) {
            Ok(tray) => Some(tray),
            Err(error) => {
                eprintln!("Pocket Pal could not create its system tray icon: {error}");
                None
            }
        };
        #[cfg(target_os = "windows")]
        if let Err(error) = ensure_desktop_shortcut() {
            eprintln!("Pocket Pal could not create a desktop shortcut: {error}");
        }
        #[cfg(target_os = "windows")]
        if save.run_at_startup {
            if let Err(error) = set_run_at_startup(true) {
                eprintln!("Pocket Pal could not restore its startup setting: {error}");
            }
        }
        Self {
            save,
            tab: Tab::Buddy,
            game: None,
            notice: String::new(),
            notice_time: 0.0,
            floating: false,
            main_hidden_for_float: false,
            floating_actions_open: false,
            care_task: None,
            celebration_time: 0.0,
            voice_line: None,
            speech_bubble_time: 0.0,
            hunger_voice_triggered: false,
            voice_line_index: 0,
            floating_overlay,
            chat: ChatSession {
                messages: Vec::new(),
                input: String::new(),
                key_input: String::new(),
                checked_provider: None,
                saved_key: false,
                pending: false,
                reply_receiver: None,
                error: None,
            },
            #[cfg(target_os = "windows")]
            tray,
            #[cfg(target_os = "windows")]
            tray_hidden: false,
            #[cfg(target_os = "windows")]
            tray_restore_pending: false,
        }
    }

    fn pet_needs(&self) -> PetNeeds {
        self.save
            .needs
            .get(&self.save.selected_pet)
            .copied()
            .unwrap_or_default()
    }

    fn persist(&mut self) {
        if let Err(error) = store_save(&self.save) {
            eprintln!("Pocket Pal could not save progress: {error}");
            self.notice = "Couldn't save progress. Check the app data folder.".to_owned();
            self.notice_time = 5.0;
        }
    }

    fn announce(&mut self, message: impl Into<String>) {
        self.notice = message.into();
        self.notice_time = 2.8;
    }

    fn set_floating(&mut self, floating: bool) {
        self.floating = floating;
        self.main_hidden_for_float = floating;
        self.floating_actions_open = false;
        match self.floating_overlay.lock() {
            Ok(mut overlay) => {
                overlay.actions_open = false;
                if !floating {
                    match overlay.chat.lock() {
                        Ok(mut chat) => chat.open = false,
                        Err(_) => eprintln!(
                            "Pocket Pal could not close floating chat because its state lock was poisoned"
                        ),
                    }
                }
            }
            Err(_) => eprintln!(
                "Pocket Pal could not update floating controls because their state lock was poisoned"
            ),
        }
    }

    fn feed(&mut self) {
        self.start_care_task(CareTaskKind::Snack);
    }

    fn nap(&mut self) {
        self.start_care_task(CareTaskKind::Nap);
    }

    fn start_care_task(&mut self, kind: CareTaskKind) {
        if self.care_task.is_some() {
            return;
        }
        if kind == CareTaskKind::Snack {
            self.hunger_voice_triggered = true;
        }
        let pet_name = self.save.selected_pet.name();
        let task = CareTask::new(kind);
        let message = match kind {
            CareTaskKind::Snack => format!("{pet_name} is enjoying a berry snack…"),
            CareTaskKind::Nap => format!("{pet_name} is settling in for a nap…"),
        };
        let duration = task.duration;
        self.care_task = Some(task);
        self.announce(message);
        self.notice_time = duration;
    }

    fn finish_care_task(&mut self, kind: CareTaskKind) {
        let needs = self.save.needs.entry(self.save.selected_pet).or_default();
        needs.joy = needs.joy.saturating_add(35).min(100);
        let message = match kind {
            CareTaskKind::Snack => {
                needs.fullness = needs.fullness.saturating_add(24).min(100);
                format!(
                    "{} finished their snack, feeling happy!",
                    self.save.selected_pet.name()
                )
            }
            CareTaskKind::Nap => {
                needs.energy = needs.energy.saturating_add(28).min(100);
                format!(
                    "{} woke up happy and refreshed!",
                    self.save.selected_pet.name()
                )
            }
        };
        self.celebration_time = 3.0;
        self.announce(message);
        self.persist();
    }

    fn begin_play(&mut self) {
        self.game = Some(StarGame::new());
    }

    fn catch_star(&mut self) {
        let mut finished = false;
        if let Some(game) = &mut self.game {
            game.score = game.score.saturating_add(1);
            game.move_target();
            game.move_timer = 0.45;
            finished = game.score >= 5;
        }
        if finished {
            self.game = None;
            self.save.coins += 8;
            self.save.play_count += 1;
            let needs = self.save.needs.entry(self.save.selected_pet).or_default();
            needs.joy = needs.joy.saturating_add(24).min(100);
            needs.energy = needs.energy.saturating_sub(8);
            self.announce("Five stars! +8 buttons and a very happy buddy!");
            self.persist();
        }
    }

    fn unlock_pet(&mut self, pet: PetKind) {
        let cost = pet.unlock_cost();
        if self.save.owned_pets.contains(&pet) || self.save.coins < cost {
            return;
        }
        self.save.coins -= cost;
        self.save.owned_pets.insert(pet);
        self.save.needs.insert(pet, PetNeeds::default());
        self.save.selected_pet = pet;
        self.announce(format!("{} joined your little home!", pet.name()));
        self.persist();
    }

    fn buy_decoration(&mut self, item: Decoration) {
        if self.save.unlocked_decor.contains(&item) || self.save.coins < item.cost() {
            return;
        }
        self.save.coins -= item.cost();
        self.save.unlocked_decor.insert(item);
        self.save.placed_decor.insert(item);
        self.announce(format!("{} added to your room!", item.name()));
        self.persist();
    }

    fn toggle_decoration(&mut self, item: Decoration) {
        if !self.save.unlocked_decor.contains(&item) {
            return;
        }
        if !self.save.placed_decor.remove(&item) {
            self.save.placed_decor.insert(item);
        }
        self.persist();
    }

    fn update(&mut self, ctx: &egui::Context, dt: f32) {
        self.poll_chat_reply(ctx);
        #[cfg(target_os = "windows")]
        self.update_tray(ctx);

        self.notice_time = (self.notice_time - dt).max(0.0);
        self.celebration_time = (self.celebration_time - dt).max(0.0);
        self.speech_bubble_time = (self.speech_bubble_time - dt).max(0.0);
        if self.speech_bubble_time == 0.0 {
            self.voice_line = None;
        }

        let fullness = self.pet_needs().fullness;
        if fullness >= 55 {
            self.hunger_voice_triggered = false;
        } else if fullness <= 30
            && !self.hunger_voice_triggered
            && self.care_task.is_none()
            && self.game.is_none()
        {
            self.hunger_voice_triggered = true;
            let line = match self.voice_line_index % 3 {
                0 => "Could I have a snack, please?",
                1 => "My tummy is rumbling. Snack time?",
                _ => "A little berry would be lovely!",
            }
            .to_owned();
            self.voice_line_index = self.voice_line_index.wrapping_add(1);
            self.voice_line = Some(line.clone());
            self.speech_bubble_time = 4.5;
            if let Err(error) = speak_pet_line(&line) {
                eprintln!("Pocket Pal could not play a pet voice line: {error}");
            }
        }

        let completed_task = if let Some(task) = &mut self.care_task {
            task.remaining = (task.remaining - dt).max(0.0);
            (task.remaining <= 0.0).then_some(task.kind)
        } else {
            None
        };
        if let Some(kind) = completed_task {
            self.care_task = None;
            self.finish_care_task(kind);
        }
        if let Some(game) = &mut self.game {
            game.time_left = (game.time_left - dt).max(0.0);
            game.move_timer = (game.move_timer - dt).max(0.0);
            if game.move_timer <= 0.0 {
                game.move_target();
                game.move_timer = 1.1;
            }
            if game.time_left <= 0.0 {
                self.game = None;
                self.announce("Time's up! Your buddy still loved playing.");
            }
        }
        ctx.request_repaint_after(Duration::from_millis(32));
    }

    #[cfg(target_os = "windows")]
    fn update_tray(&mut self, ctx: &egui::Context) {
        let minimized = ctx.input(|input| input.viewport().minimized == Some(true));
        if !minimized {
            self.tray_restore_pending = false;
        }
        if minimized && !self.tray_hidden && !self.tray_restore_pending && self.tray.is_some() {
            self.tray_hidden = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
    }

    fn update_floating_overlay(&mut self, ctx: &egui::Context) {
        let mut should_persist = false;
        if let Ok(mut overlay) = self.floating_overlay.lock() {
            if overlay.exit_requested {
                self.floating = false;
                self.main_hidden_for_float = false;
                self.floating_actions_open = false;
                overlay.actions_open = false;
                overlay.exit_requested = false;
                overlay.restore_requested = false;
                if let Ok(mut chat) = overlay.chat.lock() {
                    chat.open = false;
                }
                #[cfg(target_os = "windows")]
                {
                    self.tray_hidden = false;
                    self.tray_restore_pending = true;
                }
                ctx.send_viewport_cmd_to(
                    egui::ViewportId::ROOT,
                    egui::ViewportCommand::Minimized(false),
                );
                ctx.send_viewport_cmd_to(
                    egui::ViewportId::ROOT,
                    egui::ViewportCommand::Visible(true),
                );
                ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Focus);
                ctx.request_repaint();
                ctx.request_repaint_after(Duration::from_millis(100));
            }
            if overlay.restore_requested {
                #[cfg(target_os = "windows")]
                {
                    self.tray_hidden = false;
                    self.tray_restore_pending = true;
                }
                if self.floating {
                    self.main_hidden_for_float = false;
                    self.floating_actions_open = true;
                    overlay.actions_open = true;
                }
                overlay.restore_requested = false;
                show_root_viewport(ctx);
                ctx.request_repaint();
                ctx.request_repaint_after(Duration::from_millis(100));
            }
            if overlay.dirty && !self.main_hidden_for_float {
                let old_needs = self.save.needs.get(&overlay.pet).copied();
                let old_coins = self.save.coins;
                let old_play_count = self.save.play_count;
                self.save.needs.insert(overlay.pet, overlay.needs);
                if overlay.reward_pending {
                    self.save.coins += 8;
                    self.save.play_count += 1;
                    overlay.reward_pending = false;
                }
                should_persist = old_needs != Some(overlay.needs)
                    || old_coins != self.save.coins
                    || old_play_count != self.save.play_count;
                self.care_task = overlay.care_task.map(|(kind, progress)| {
                    let mut task = CareTask::new(kind);
                    task.remaining = task.duration * (1.0 - progress.clamp(0.0, 1.0));
                    task
                });
                self.celebration_time = if overlay.celebrating {
                    overlay.celebration_time.max(3.0)
                } else {
                    0.0
                };
                self.game = overlay.game.map(|(score, time_left, target)| {
                    let mut game = StarGame::new();
                    game.score = score;
                    game.time_left = time_left;
                    game.target = target;
                    game.move_timer = overlay.game_move_timer;
                    game.target_step = overlay.game_target_step;
                    game
                });
                overlay.dirty = false;
            } else if self.main_hidden_for_float {
                return;
            }

            overlay.pet = self.save.selected_pet;
            overlay.needs = self.pet_needs();
            overlay.care_task = self
                .care_task
                .as_ref()
                .map(|task| (task.kind, task.progress()));
            overlay.celebrating = self.celebration_time > 0.0;
            overlay.celebration_time = self.celebration_time;
            overlay.voice_line = self
                .voice_line
                .as_ref()
                .filter(|_| self.speech_bubble_time > 0.0)
                .cloned();
            overlay.game = self
                .game
                .as_ref()
                .map(|game| (game.score, game.time_left, game.target));
            if let Some(game) = &self.game {
                overlay.game_move_timer = game.move_timer;
                overlay.game_target_step = game.target_step;
            }
            if !self.floating {
                overlay.actions_open = false;
            }
        } else {
            eprintln!(
                "Pocket Pal could not update floating pet state because its lock was poisoned"
            );
        }
        if should_persist {
            self.persist();
        }
    }

    fn poll_chat_reply(&mut self, ctx: &egui::Context) {
        let received = self
            .chat
            .reply_receiver
            .as_ref()
            .map(mpsc::Receiver::try_recv);
        match received {
            Some(Ok(Ok(reply))) => {
                self.chat.messages.push(chat::Message {
                    speaker: chat::Speaker::Pet,
                    text: reply,
                });
                self.chat.reply_receiver = None;
                self.chat.pending = false;
                self.chat.error = None;
            }
            Some(Ok(Err(error))) => {
                self.chat.reply_receiver = None;
                self.chat.pending = false;
                self.chat.error = Some(error);
            }
            Some(Err(mpsc::TryRecvError::Disconnected)) => {
                self.chat.reply_receiver = None;
                self.chat.pending = false;
                self.chat.error = Some("The chat request stopped unexpectedly.".to_owned());
            }
            Some(Err(mpsc::TryRecvError::Empty)) | None => {}
        }
        if self.chat.pending {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn send_chat_message(&mut self, ctx: &egui::Context) {
        let text = self.chat.input.trim().to_owned();
        if text.is_empty() || self.chat.pending {
            return;
        }
        self.chat.messages.push(chat::Message {
            speaker: chat::Speaker::User,
            text,
        });
        if self.chat.messages.len() > 20 {
            let excess = self.chat.messages.len() - 20;
            self.chat.messages.drain(..excess);
        }

        let provider = self.save.chat_provider;
        let pet_name = self.save.selected_pet.name().to_owned();
        let history = self.chat.messages.clone();
        let (sender, receiver) = mpsc::channel();
        self.chat.reply_receiver = Some(receiver);
        self.chat.pending = true;
        self.chat.input.clear();
        self.chat.error = None;
        std::thread::spawn(move || {
            let result = chat::load_key(provider)
                .and_then(|key| chat::send_message(provider, &key, &pet_name, &history));
            let _ = sender.send(result);
        });
        ctx.request_repaint();
    }

    fn show_header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar")
            .exact_height(56.0)
            .frame(
                egui::Frame::default()
                    .fill(APP_PANEL)
                    .inner_margin(egui::Margin::symmetric(16, 7)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    round_icon_badge(ui, "*", APP_ACCENT, 30.0);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("POCKET PAL")
                                .font(egui::FontId::new(17.0, egui::FontFamily::Proportional))
                                .strong()
                                .color(APP_TEXT),
                        );
                        ui.label(
                            egui::RichText::new("DESKTOP COMPANION")
                                .size(9.0)
                                .color(APP_MUTED),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if small_button(ui, "FLOAT", APP_ACCENT).clicked() {
                            self.set_floating(true);
                        }
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new(format!("{}  BUTTONS", self.save.coins))
                                .size(11.0)
                                .strong()
                                .color(APP_TEXT),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(format!("{} PALS", self.save.owned_pets.len()))
                                .size(10.0)
                                .color(APP_MUTED),
                        );
                    });
                });
            });
    }

    fn show_room(&self, ui: &mut egui::Ui, time: f64) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("YOUR ROOM")
                    .size(11.0)
                    .strong()
                    .color(APP_MUTED),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("a place to just be")
                    .size(11.0)
                    .color(APP_MUTED),
            );
        });
        ui.add_space(4.0);
        let available = ui.available_size();
        let desired = Vec2::new(available.x.max(360.0), (available.y - 78.0).max(280.0));
        let (room, _) = ui.allocate_exact_size(desired, Sense::hover());
        paint_room(
            ui.painter(),
            room,
            &self.save,
            time,
            self.celebration_time > 0.0,
            self.care_task
                .as_ref()
                .map(|task| (task.kind, task.progress())),
            self.voice_line
                .as_deref()
                .filter(|_| self.speech_bubble_time > 0.0),
        );
        ui.add_space(4.0);
        ui.horizontal_centered(|ui| {
            ui.label(
                egui::RichText::new("No rush, no chores. Just hang out together.")
                    .size(10.0)
                    .color(APP_MUTED),
            );
        });
    }

    fn show_buddy_tab(&mut self, ui: &mut egui::Ui) {
        let pet = self.save.selected_pet;
        let needs = self.pet_needs();
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(format!("{}  ·  {}", pet.name(), pet.species()))
                    .size(19.0)
                    .strong()
                    .color(APP_TEXT),
            );
            ui.label(
                egui::RichText::new(pet_mood(needs, self.celebration_time > 0.0))
                    .size(12.0)
                    .color(APP_MUTED),
            );
            ui.add_space(12.0);
            stat_bar(ui, "FULL", needs.fullness, Color32::from_gray(220));
            stat_bar(ui, "JOY", needs.joy, Color32::from_gray(175));
            stat_bar(ui, "ENERGY", needs.energy, Color32::from_gray(125));
            ui.add_space(7.0);
            let care_enabled = self.care_task.is_none();
            ui.add_enabled_ui(care_enabled, |ui| {
                ui.horizontal(|ui| {
                    if chunky_button(ui, "Feed", APP_FEED).clicked() {
                        self.feed();
                    }
                    if chunky_button(ui, "Nap", APP_NAP).clicked() {
                        self.nap();
                    }
                });
            });
            if let Some(task) = &self.care_task {
                ui.add_space(6.0);
                let (line, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 2.0), Sense::hover());
                ui.painter().line_segment(
                    [line.left_center(), line.right_center()],
                    Stroke::new(1.0_f32, APP_SURFACE_HI),
                );
                ui.painter().line_segment(
                    [
                        line.left_center(),
                        Pos2::new(
                            line.left() + line.width() * task.progress().clamp(0.0, 1.0),
                            line.center().y,
                        ),
                    ],
                    Stroke::new(
                        2.0_f32,
                        match task.kind {
                            CareTaskKind::Snack => MELON,
                            CareTaskKind::Nap => SKY,
                        },
                    ),
                );
            }
            ui.add_space(8.0);
            if chunky_button(ui, "Play Star Catcher", APP_PLAY).clicked() {
                self.begin_play();
            }
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Little moments")
                    .size(11.0)
                    .strong()
                    .color(APP_MUTED),
            );
            ui.label(
                egui::RichText::new(format!("{} star games won", self.save.play_count))
                    .size(12.0)
                    .color(APP_TEXT),
            );
        });
    }

    fn show_collection_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("MEET THE PALS")
                .size(17.0)
                .strong()
                .color(APP_TEXT),
        );
        ui.label(
            egui::RichText::new("Two friends to start; two more to find.")
                .size(11.0)
                .color(APP_MUTED),
        );
        ui.add_space(8.0);
        for pet in PetKind::ALL {
            let owned = self.save.owned_pets.contains(&pet);
            egui::Frame::default()
                .fill(if self.save.selected_pet == pet && owned {
                    APP_SURFACE_HI
                } else {
                    APP_PANEL
                })
                .stroke(Stroke::new(1.0_f32, APP_BORDER))
                .inner_margin(egui::Margin::symmetric(8, 5))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        draw_pet_icon(ui, pet);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(pet.name())
                                    .size(14.0)
                                    .strong()
                                    .color(APP_TEXT),
                            );
                            ui.label(
                                egui::RichText::new(pet.species())
                                    .size(9.0)
                                    .color(APP_MUTED),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if owned {
                                let label = if self.save.selected_pet == pet {
                                    "HERE"
                                } else {
                                    "VISIT"
                                };
                                if small_button(ui, label, APP_ACCENT).clicked() {
                                    self.save.selected_pet = pet;
                                    self.persist();
                                }
                            } else {
                                let cost = pet.unlock_cost();
                                let label = format!("{cost} BTN");
                                if small_button(ui, &label, APP_PLAY).clicked() {
                                    self.unlock_pet(pet);
                                }
                            }
                        });
                    });
                });
            ui.add_space(6.0);
        }
    }

    fn show_decor_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("LITTLE FINDS")
                .size(17.0)
                .strong()
                .color(APP_TEXT),
        );
        ui.label(
            egui::RichText::new("Make your room feel like yours.")
                .size(11.0)
                .color(APP_MUTED),
        );
        ui.add_space(12.0);
        for item in Decoration::ALL {
            let unlocked = self.save.unlocked_decor.contains(&item);
            let placed = self.save.placed_decor.contains(&item);
            egui::Frame::default()
                .fill(APP_PANEL)
                .stroke(Stroke::new(1.0_f32, APP_BORDER))
                .inner_margin(egui::Margin::symmetric(8, 5))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        round_icon_badge(ui, decoration_glyph(item), APP_LAVENDER, 40.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(item.name())
                                    .size(12.0)
                                    .strong()
                                    .color(APP_TEXT),
                            );
                            ui.label(
                                egui::RichText::new(item.description())
                                    .size(10.0)
                                    .color(APP_MUTED),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if unlocked {
                                if small_button(
                                    ui,
                                    if placed { "PUT AWAY" } else { "PLACE" },
                                    APP_ACCENT,
                                )
                                .clicked()
                                {
                                    self.toggle_decoration(item);
                                }
                            } else if small_button(ui, &format!("{} BTN", item.cost()), APP_PLAY)
                                .clicked()
                            {
                                self.buy_decoration(item);
                            }
                        });
                    });
                });
            ui.add_space(6.0);
        }
    }

    fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("FLOATING PET")
                .size(17.0)
                .strong()
                .color(APP_TEXT),
        );
        ui.label(
            egui::RichText::new("Choose where your companion sits on your desktop.")
                .size(11.0)
                .color(APP_MUTED),
        );
        ui.add_space(14.0);

        if ui
            .radio(self.save.floating_on_top, "Always on top of apps")
            .clicked()
        {
            self.save.floating_on_top = true;
            self.persist();
        }
        ui.add_space(8.0);
        if ui
            .radio(!self.save.floating_on_top, "On desktop, behind apps")
            .clicked()
        {
            self.save.floating_on_top = false;
            self.persist();
        }
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(
                "The desktop option keeps the pet behind regular app windows. Switch to the desktop to see it.",
            )
            .size(11.0)
            .color(APP_MUTED),
        );
        #[cfg(target_os = "windows")]
        {
            ui.add_space(18.0);
            ui.separator();
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new("WINDOWS")
                    .size(15.0)
                    .strong()
                    .color(APP_TEXT),
            );
            let mut run_at_startup = self.save.run_at_startup;
            if ui
                .checkbox(&mut run_at_startup, "Run Pocket Pal when I sign in")
                .changed()
            {
                match set_run_at_startup(run_at_startup) {
                    Ok(()) => {
                        self.save.run_at_startup = run_at_startup;
                        self.persist();
                    }
                    Err(error) => {
                        self.announce(format!("Could not update startup setting: {error}"));
                    }
                }
            }
            ui.label(
                egui::RichText::new(
                    "A desktop shortcut is created automatically when Pocket Pal starts.",
                )
                .size(11.0)
                .color(APP_MUTED),
            );
        }
        #[cfg(not(target_os = "windows"))]
        ui.label(
            egui::RichText::new(
                "Automatic startup and desktop shortcuts are available on Windows.",
            )
            .size(11.0)
            .color(APP_MUTED),
        );
    }

    fn show_chat_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("CHAT WITH {}", self.save.selected_pet.name()))
                    .size(17.0)
                    .strong()
                    .color(APP_TEXT),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if small_button(ui, "Back to pal", APP_SURFACE_HI).clicked() {
                    self.tab = Tab::Buddy;
                }
            });
        });
        ui.label(
            egui::RichText::new("A little chat with your desktop buddy.")
                .size(11.0)
                .color(APP_MUTED),
        );
        ui.add_space(8.0);

        let provider_before = self.save.chat_provider;
        egui::ComboBox::from_id_salt("chat_provider")
            .selected_text(self.save.chat_provider.name())
            .show_ui(ui, |ui| {
                for provider in chat::Provider::ALL {
                    ui.selectable_value(&mut self.save.chat_provider, provider, provider.name());
                }
            });
        if self.save.chat_provider != provider_before {
            self.chat.checked_provider = None;
            self.chat.error = None;
            self.persist();
        }

        if self.chat.checked_provider != Some(self.save.chat_provider) {
            self.chat.checked_provider = Some(self.save.chat_provider);
            self.chat.saved_key = match chat::has_saved_key(self.save.chat_provider) {
                Ok(saved) => saved,
                Err(error) => {
                    self.chat.error = Some(error);
                    false
                }
            };
        }

        ui.label(
            egui::RichText::new(
                "Your messages go directly to the selected provider and may use your API billing.",
            )
            .size(10.0)
            .color(APP_MUTED),
        );
        if self.chat.saved_key {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("API key saved in the OS credential store.")
                        .size(10.0)
                        .color(APP_ACCENT),
                );
                if small_button(ui, "Forget key", APP_SURFACE_HI).clicked() {
                    match chat::delete_key(self.save.chat_provider) {
                        Ok(()) => {
                            self.chat.saved_key = false;
                            self.chat.key_input.clear();
                            self.chat.error = None;
                        }
                        Err(error) => self.chat.error = Some(error),
                    }
                }
            });
        } else {
            ui.label(
                egui::RichText::new("Enter your API key once; Pocket Pal stores it securely.")
                    .size(10.0)
                    .color(APP_MUTED),
            );
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.chat.key_input)
                        .password(true)
                        .hint_text("Paste API key")
                        .desired_width(ui.available_width() - 112.0),
                );
                if small_button(ui, "Save key", APP_ACCENT).clicked() {
                    let key = self.chat.key_input.trim().to_owned();
                    if key.is_empty() {
                        self.chat.error = Some("Enter an API key first.".to_owned());
                    } else {
                        match chat::save_key(self.save.chat_provider, &key) {
                            Ok(()) => {
                                self.chat.key_input.clear();
                                self.chat.saved_key = true;
                                self.chat.error = None;
                            }
                            Err(error) => self.chat.error = Some(error),
                        }
                    }
                }
            });
        }

        if let Some(error) = &self.chat.error {
            ui.label(
                egui::RichText::new(error)
                    .size(10.0)
                    .color(Color32::from_rgb(255, 139, 165)),
            );
        }

        ui.add_space(6.0);
        let history_height = (ui.available_height() - 48.0).clamp(120.0, 360.0);
        egui::ScrollArea::vertical()
            .id_salt("pet_chat_history")
            .stick_to_bottom(true)
            .max_height(history_height)
            .show(ui, |ui| {
                if self.chat.messages.is_empty() {
                    ui.label(
                        egui::RichText::new("Say hi to get the conversation started.")
                            .size(11.0)
                            .color(APP_MUTED),
                    );
                }
                for message in &self.chat.messages {
                    let (alignment, fill, speaker) = match message.speaker {
                        chat::Speaker::User => (egui::Align::Max, APP_SURFACE_HI, "YOU"),
                        chat::Speaker::Pet => (
                            egui::Align::Min,
                            Color32::from_rgb(49, 31, 72),
                            self.save.selected_pet.name(),
                        ),
                    };
                    ui.with_layout(egui::Layout::top_down(alignment), |ui| {
                        egui::Frame::default()
                            .fill(fill)
                            .corner_radius(egui::CornerRadius::same(7))
                            .inner_margin(egui::Margin::symmetric(8, 6))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(speaker)
                                        .size(8.0)
                                        .strong()
                                        .color(APP_ACCENT),
                                );
                                ui.label(
                                    egui::RichText::new(&message.text)
                                        .size(11.0)
                                        .color(APP_TEXT),
                                );
                            });
                    });
                    ui.add_space(4.0);
                }
                if self.chat.pending {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} is thinking...",
                            self.save.selected_pet.name()
                        ))
                        .size(10.0)
                        .color(APP_MUTED),
                    );
                }
            });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let input = ui.add_enabled(
                self.chat.saved_key && !self.chat.pending,
                egui::TextEdit::singleline(&mut self.chat.input)
                    .char_limit(1000)
                    .hint_text("Write a message...")
                    .desired_width(ui.available_width() - 70.0),
            );
            let send = small_button(
                ui,
                if self.chat.pending { "..." } else { "Send" },
                APP_ACCENT,
            );
            if send.clicked()
                || (input.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)))
            {
                self.send_chat_message(ctx);
            }
        });
    }

    fn show_floating_pet(&mut self, ctx: &egui::Context, time: f64) {
        let progress = ctx.animate_bool_with_time(
            egui::Id::new("floating_pet_actions"),
            self.floating_actions_open,
            0.24,
        );
        egui::Area::new(egui::Id::new("floating_pet_card"))
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                let (canvas, _) = ui.allocate_exact_size(Vec2::new(256.0, 256.0), Sense::hover());
                let pet = self.save.selected_pet;
                let needs = self.pet_needs();
                let bounce = ((time * 2.7).sin() * 1.5) as f32;
                let pet_origin = Pos2::new(canvas.left() + 64.0, canvas.top() + 64.0 + bounce);
                let response = ui.interact(
                    Rect::from_min_size(pet_origin, Vec2::splat(128.0)),
                    egui::Id::new("floating_pet_click"),
                    Sense::click_and_drag(),
                );
                if response.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if response.clicked() {
                    self.floating_actions_open = !self.floating_actions_open;
                }
                let mood = pet_mood_state_for(needs, self.celebration_time > 0.0);
                paint_pet_sprite(
                    ui.painter(),
                    pet_origin,
                    8.0,
                    pet,
                    mood,
                    self.care_task
                        .as_ref()
                        .map(|task| (task.kind, task.progress())),
                    time,
                );
                if let Some(line) = self
                    .voice_line
                    .as_deref()
                    .filter(|_| self.speech_bubble_time > 0.0)
                {
                    paint_pixel_bubble(
                        ui.painter(),
                        Pos2::new(canvas.center().x, pet_origin.y - 5.0),
                        line,
                        226.0,
                    );
                }

                if self.care_task.is_some() {
                    if let Some(task) = &self.care_task {
                        let task_color = match task.kind {
                            CareTaskKind::Snack => MELON,
                            CareTaskKind::Nap => SKY,
                        };
                        let progress_line = Rect::from_min_max(
                            Pos2::new(canvas.center().x - 44.0, canvas.bottom() - 13.0),
                            Pos2::new(canvas.center().x + 44.0, canvas.bottom() - 11.0),
                        );
                        ui.painter().line_segment(
                            [progress_line.left_center(), progress_line.right_center()],
                            Stroke::new(1.0_f32, with_alpha(APP_SURFACE_HI, 180)),
                        );
                        ui.painter().line_segment(
                            [
                                progress_line.left_center(),
                                Pos2::new(
                                    progress_line.left()
                                        + progress_line.width() * task.progress().clamp(0.0, 1.0),
                                    progress_line.center().y,
                                ),
                            ],
                            Stroke::new(2.0_f32, with_alpha(task_color, 230)),
                        );
                    }
                }

                if self.floating_actions_open || progress > 0.0 {
                    let alpha = (progress * 255.0) as u8;
                    let actions = [
                        (FLOAT_FEED, Vec2::new(128.0, 19.0)),
                        (FLOAT_NAP, Vec2::new(224.0, 90.0)),
                        (FLOAT_PLAY, Vec2::new(224.0, 218.0)),
                        (FLOAT_CHAT, Vec2::new(32.0, 218.0)),
                        (Color32::from_gray(65), Vec2::new(32.0, 90.0)),
                    ];
                    for (index, (color, center)) in actions.into_iter().enumerate() {
                        let animation_offset = (1.0 - progress) * 10.0;
                        let direction = (center - Vec2::splat(128.0)).normalized();
                        let button_center = canvas.min + center + direction * animation_offset;
                        let rect = Rect::from_center_size(button_center, Vec2::splat(40.0));
                        let response = ui.interact(
                            rect,
                            egui::Id::new(("floating_pet_action", index)),
                            Sense::click(),
                        );
                        let painter = ui.painter();
                        painter.rect_filled(
                            rect,
                            egui::CornerRadius::same(12),
                            with_alpha(color, alpha),
                        );
                        painter.rect_stroke(
                            rect,
                            egui::CornerRadius::same(12),
                            Stroke::new(1.0_f32, with_alpha(APP_BORDER, alpha)),
                            egui::StrokeKind::Middle,
                        );
                        paint_floating_action_icon(
                            painter,
                            button_center,
                            index,
                            with_alpha(APP_TEXT, alpha),
                        );
                        if response.clicked() {
                            match index {
                                0 => self.feed(),
                                1 => self.nap(),
                                2 => {
                                    self.begin_play();
                                    self.floating_actions_open = false;
                                }
                                3 => {
                                    self.floating_actions_open = false;
                                    let chat_state = match self.floating_overlay.lock() {
                                        Ok(overlay) => Arc::clone(&overlay.chat),
                                        Err(_) => {
                                            eprintln!("Pocket Pal could not open floating chat because its state lock was poisoned");
                                            return;
                                        }
                                    };
                                    toggle_floating_chat(ctx, &chat_state);
                                }
                                4 => self.set_floating(false),
                                _ => unreachable!(),
                            }
                        }
                    }
                }
            });
        if let Ok(overlay) = self.floating_overlay.lock() {
            show_floating_chat(ctx, Arc::clone(&overlay.chat), self.save.selected_pet);
        } else {
            eprintln!(
                "Pocket Pal could not render floating chat because its state lock was poisoned"
            );
        }
    }

    fn show_floating_star_game(&mut self, ctx: &egui::Context) {
        egui::Area::new(egui::Id::new("floating_star_game"))
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                let (screen, _) = ui.allocate_exact_size(Vec2::new(256.0, 256.0), Sense::hover());
                let painter = ui.painter();
                for index in 0..18 {
                    let x = ((index * 197 + 41) % 251) as f32;
                    let y = ((index * 113 + 29) % 251) as f32;
                    let radius = if index % 4 == 0 { 5.0 } else { 2.5 };
                    let center = Pos2::new(screen.left() + x, screen.top() + y);
                    painter.add(egui::Shape::convex_polygon(
                        star_points(center, radius, radius * 0.45),
                        with_alpha(LEMON, if index % 4 == 0 { 150 } else { 85 }),
                        Stroke::NONE,
                    ));
                }

                let pet = self.save.selected_pet;
                let pet_needs = self.pet_needs();
                let pet_origin = Pos2::new(screen.center().x - 48.0, screen.top() + 88.0);
                paint_pet_sprite(
                    painter,
                    pet_origin,
                    6.0,
                    pet,
                    pet_mood_state_for(pet_needs, self.celebration_time > 0.0),
                    self.care_task
                        .as_ref()
                        .map(|task| (task.kind, task.progress())),
                    ctx.input(|input| input.time),
                );

                let (score, time_left, target) = self
                    .game
                    .as_ref()
                    .map(|game| (game.score, game.time_left, game.target))
                    .unwrap_or((0, 0.0, (0.5, 0.5)));
                painter.text(
                    Pos2::new(screen.center().x, screen.top() + 12.0),
                    egui::Align2::CENTER_TOP,
                    format!("{score} / 5  ·  {time_left:.0}s"),
                    egui::FontId::proportional(12.0),
                    APP_TEXT,
                );

                let star_center = Pos2::new(
                    screen.left() + 22.0 + target.0 * (screen.width() - 44.0).max(1.0),
                    screen.top() + 22.0 + target.1 * (screen.height() - 44.0).max(1.0),
                );
                let target_rect = Rect::from_center_size(star_center, Vec2::splat(46.0));
                let target_response = ui.interact(
                    target_rect,
                    egui::Id::new("floating_star_target"),
                    Sense::click(),
                );
                let star_radius = if target_response.hovered() {
                    17.0
                } else {
                    14.0
                };
                painter.add(egui::Shape::convex_polygon(
                    star_points(star_center, star_radius, star_radius * 0.46),
                    LEMON,
                    Stroke::new(1.0_f32, Color32::from_rgb(255, 249, 218)),
                ));
                if target_response.clicked() {
                    self.catch_star();
                }

                let exit_rect = Rect::from_min_size(
                    Pos2::new(screen.right() - 30.0, screen.top() + 4.0),
                    Vec2::splat(24.0),
                );
                let exit_response = ui.interact(
                    exit_rect,
                    egui::Id::new("floating_star_exit"),
                    Sense::click(),
                );
                paint_floating_action_icon(painter, exit_rect.center(), 4, APP_TEXT);
                if exit_response.clicked() {
                    self.game = None;
                }
            });
    }

    fn show_star_game(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("STAR CATCHER")
                .size(17.0)
                .strong()
                .color(APP_TEXT),
        );
        ui.label(
            egui::RichText::new("Catch five twinkles before they float away.")
                .size(11.0)
                .color(APP_MUTED),
        );
        ui.add_space(10.0);
        let Some(game) = &self.game else {
            egui::Frame::default()
                .fill(APP_SURFACE)
                .corner_radius(egui::CornerRadius::same(10))
                .stroke(Stroke::new(1.0_f32, APP_BORDER))
                .inner_margin(egui::Margin::symmetric(14, 14))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("A tiny game for a tiny pal.")
                            .size(13.0)
                            .strong()
                            .color(APP_TEXT),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Win 5 stars to earn 8 buttons!")
                            .size(11.0)
                            .color(APP_MUTED),
                    );
                    ui.add_space(12.0);
                    if chunky_button(ui, "*  Let's play!", APP_PLAY).clicked() {
                        self.begin_play();
                    }
                });
            return;
        };
        let score = game.score;
        let time_left = game.time_left;
        let target = game.target;
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("STARS  {score} / 5"))
                    .size(12.0)
                    .strong()
                    .color(APP_TEXT),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{time_left:.0}s"))
                        .size(12.0)
                        .strong()
                        .color(APP_MUTED),
                );
            });
        });
        ui.add_space(5.0);
        let (field, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 156.0), Sense::hover());
        ui.painter().rect_filled(field, 3.0, APP_SURFACE);
        for i in 0..14 {
            let x = field.left() + ((i * 37 + 17) % field.width().max(1.0) as usize) as f32;
            let y = field.top() + ((i * 23 + 11) % 145) as f32;
            ui.painter().rect_filled(
                Rect::from_min_size(Pos2::new(x, y), Vec2::splat(2.0)),
                0.0,
                APP_SURFACE_HI,
            );
        }
        let center = Pos2::new(
            field.left() + 22.0 + target.0 * (field.width() - 44.0).max(1.0),
            field.top() + 23.0 + target.1 * (field.height() - 46.0).max(1.0),
        );
        let star_rect = Rect::from_center_size(center, Vec2::splat(38.0));
        if ui
            .put(
                star_rect,
                egui::Button::new(egui::RichText::new("*").size(27.0).color(LEMON)),
            )
            .clicked()
        {
            self.catch_star();
        }
        ui.add_space(8.0);
        if small_button(ui, "Leave game", APP_SURFACE_HI).clicked() {
            self.game = None;
        }
    }
}

impl eframe::App for PocketPal {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|input| input.unstable_dt).clamp(0.0, 0.1);
        self.update(ctx, dt);
        self.update_floating_overlay(ctx);
        if self.floating {
            let level = if self.save.floating_on_top {
                egui::viewport::WindowLevel::AlwaysOnTop
            } else {
                egui::viewport::WindowLevel::AlwaysOnBottom
            };
            if ctx.embed_viewports() {
                if self.game.is_some() {
                    self.show_floating_star_game(ctx);
                } else {
                    self.show_floating_pet(ctx, ctx.input(|input| input.time));
                }
            } else {
                let builder = egui::ViewportBuilder::default()
                    .with_title("Pocket Pal")
                    .with_inner_size([256.0, 256.0])
                    .with_min_inner_size([256.0, 256.0])
                    .with_max_inner_size([256.0, 256.0])
                    .with_resizable(false)
                    .with_decorations(false)
                    .with_transparent(true)
                    .with_taskbar(false)
                    .with_window_level(level);
                let overlay_state = Arc::clone(&self.floating_overlay);
                let root_ctx = ctx.clone();
                ctx.show_viewport_deferred(
                    floating_pet_viewport_id(),
                    builder,
                    move |pet_ctx, _class| {
                        pet_ctx.send_viewport_cmd(egui::ViewportCommand::Transparent(true));
                        show_floating_overlay(pet_ctx, &overlay_state, &root_ctx);
                    },
                );
                self.update_floating_overlay(ctx);
            }
            #[cfg(target_os = "windows")]
            let hide_main =
                self.main_hidden_for_float && self.tray.is_some() && !ctx.embed_viewports();
            #[cfg(not(target_os = "windows"))]
            let hide_main = false;
            if hide_main {
                #[cfg(target_os = "windows")]
                let pet_window_visible = ctx.input(|input| {
                    input
                        .raw
                        .viewports
                        .get(&floating_pet_viewport_id())
                        .and_then(|viewport| viewport.inner_rect)
                        .is_some()
                });
                #[cfg(target_os = "windows")]
                if pet_window_visible && !self.tray_hidden {
                    self.tray_hidden = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                }
                ctx.request_repaint_after(Duration::from_millis(100));
                return;
            }
        }
        self.show_header(ctx);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(APP_BG)
                    .inner_margin(egui::Margin::symmetric(12, 8)),
            )
            .show(ctx, |ui| {
                ui.columns(2, |columns| {
                    columns[0].vertical(|ui| self.show_room(ui, ctx.input(|i| i.time)));
                    columns[1].vertical(|ui| {
                        ui.horizontal(|ui| {
                            tab_button(ui, "Pal", self.tab == Tab::Buddy)
                                .clicked()
                                .then(|| self.tab = Tab::Buddy);
                            tab_button(ui, "Friends", self.tab == Tab::Collection)
                                .clicked()
                                .then(|| self.tab = Tab::Collection);
                            tab_button(ui, "Room", self.tab == Tab::Decor)
                                .clicked()
                                .then(|| self.tab = Tab::Decor);
                            tab_button(ui, "Settings", self.tab == Tab::Settings)
                                .clicked()
                                .then(|| self.tab = Tab::Settings);
                            tab_button(ui, "Chat", self.tab == Tab::Chat)
                                .clicked()
                                .then(|| self.tab = Tab::Chat);
                        });
                        ui.add_space(6.0);
                        egui::Frame::default()
                            .fill(APP_PANEL)
                            .stroke(Stroke::new(1.0_f32, APP_BORDER))
                            .inner_margin(egui::Margin::symmetric(10, 8))
                            .show(ui, |ui| match self.tab {
                                Tab::Buddy => {
                                    if self.game.is_some() {
                                        self.show_star_game(ui);
                                    } else {
                                        self.show_buddy_tab(ui);
                                    }
                                }
                                Tab::Collection => self.show_collection_tab(ui),
                                Tab::Decor => self.show_decor_tab(ui),
                                Tab::Settings => self.show_settings_tab(ui),
                                Tab::Chat => self.show_chat_tab(ui, ctx),
                            });
                        ui.add_space(5.0);
                        if self.notice_time > 0.0 {
                            egui::Frame::default()
                                .fill(APP_SURFACE_HI)
                                .corner_radius(egui::CornerRadius::same(8))
                                .inner_margin(egui::Margin::symmetric(8, 5))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(&self.notice)
                                            .size(11.0)
                                            .color(APP_TEXT),
                                    );
                                });
                        }
                        ui.add_space(3.0);
                        ui.label(
                            egui::RichText::new("Your pals never mind if you take a break.")
                                .size(10.0)
                                .color(APP_MUTED),
                        );
                    });
                });
            });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}

fn style_app(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    visuals.window_fill = APP_PANEL;
    visuals.panel_fill = APP_BG;
    visuals.window_corner_radius = egui::CornerRadius::same(4);
    visuals.menu_corner_radius = egui::CornerRadius::same(3);
    visuals.extreme_bg_color = Color32::from_rgb(5, 5, 5);
    visuals.faint_bg_color = APP_SURFACE;
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(3);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(3);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(3);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(3);
    visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, APP_TEXT);
    visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, APP_TEXT);
    visuals.widgets.hovered.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, APP_TEXT);
    visuals.widgets.active.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, APP_TEXT);
    visuals.selection.bg_fill = APP_ACCENT;
    visuals.selection.stroke = Stroke::new(1.0_f32, APP_TEXT);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(7.0, 5.0);
    style.spacing.button_padding = Vec2::new(8.0, 4.0);
    style.spacing.menu_margin = egui::Margin::same(6);
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(13.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(12.0, egui::FontFamily::Proportional),
    );
    ctx.set_style(style);
}

fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

fn star_points(center: Pos2, outer_radius: f32, inner_radius: f32) -> Vec<Pos2> {
    (0..10)
        .map(|point| {
            let angle = -std::f32::consts::FRAC_PI_2 + point as f32 * std::f32::consts::TAU / 10.0;
            let radius = if point % 2 == 0 {
                outer_radius
            } else {
                inner_radius
            };
            center + Vec2::new(angle.cos(), angle.sin()) * radius
        })
        .collect()
}

fn paint_pet_sprite(
    painter: &egui::Painter,
    origin: Pos2,
    cell: f32,
    pet: PetKind,
    mood: PetMood,
    care: Option<(CareTaskKind, f32)>,
    time: f64,
) {
    if matches!(care, Some((CareTaskKind::Nap, _))) {
        paint_sleeping_pet(painter, origin, cell, pet, time);
        return;
    }

    let (fur, cream, dark) = pet.colors();
    let chew_bob = if matches!(care, Some((CareTaskKind::Snack, _))) {
        (time as f32 * 13.0).sin().abs() * cell * 0.45
    } else {
        0.0
    };
    let pose_origin = origin + Vec2::new(0.0, -chew_bob);
    for (row, line) in pet.sprite().iter().enumerate() {
        for (column, pixel) in line.chars().enumerate() {
            let color = match pixel {
                'F' => fur,
                'D' => dark,
                'C' => cream,
                'E' => continue,
                'P' => ROSE,
                _ => continue,
            };
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(
                        pose_origin.x + column as f32 * cell,
                        pose_origin.y + row as f32 * cell,
                    ),
                    Vec2::splat(cell),
                ),
                0.0,
                color,
            );
        }
    }

    let pixel = |x: f32, y: f32, width: f32, height: f32, color: Color32| {
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(pose_origin.x + x * cell, pose_origin.y + y * cell),
                Vec2::new(width * cell, height * cell),
            ),
            0.0,
            color,
        );
    };

    match mood {
        PetMood::Sleepy => {
            pixel(3.0, 8.0, 2.0, 1.0, INK);
            pixel(11.0, 8.0, 2.0, 1.0, INK);
            pixel(7.0, 12.0, 2.0, 1.0, INK);
        }
        PetMood::Hungry => {
            pixel(3.0, 7.0, 2.0, 2.0, INK);
            pixel(11.0, 7.0, 2.0, 2.0, INK);
            pixel(7.0, 11.0, 2.0, 2.0, INK);
            pixel(7.0, 12.0, 1.0, 1.0, ROSE);
        }
        PetMood::Sad => {
            pixel(3.0, 7.0, 2.0, 2.0, INK);
            pixel(11.0, 7.0, 2.0, 2.0, INK);
            pixel(3.0, 6.0, 2.0, 1.0, INK);
            pixel(11.0, 6.0, 2.0, 1.0, INK);
            pixel(6.0, 12.0, 1.0, 1.0, INK);
            pixel(7.0, 11.0, 2.0, 1.0, INK);
            pixel(9.0, 12.0, 1.0, 1.0, INK);
        }
        PetMood::Happy => {
            pixel(3.0, 7.0, 2.0, 2.0, INK);
            pixel(11.0, 7.0, 2.0, 2.0, INK);
            pixel(6.0, 11.0, 1.0, 1.0, INK);
            pixel(7.0, 12.0, 2.0, 1.0, INK);
            pixel(9.0, 11.0, 1.0, 1.0, INK);
        }
    }

    if let Some((CareTaskKind::Snack, progress)) = care {
        paint_eating_animation(painter, pose_origin, cell, progress, time);
    }
}

fn paint_eating_animation(
    painter: &egui::Painter,
    origin: Pos2,
    cell: f32,
    progress: f32,
    time: f64,
) {
    let phase = (progress * 3.0).fract();
    let approach = (phase / 0.72).clamp(0.0, 1.0);
    let berry_x = 3.0 + approach * 4.2;
    let berry_y = 11.4 - approach * 1.2 + (phase * std::f32::consts::TAU).sin() * 0.25;
    let pixel = |x: f32, y: f32, width: f32, height: f32, color: Color32| {
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(origin.x + x * cell, origin.y + y * cell),
                Vec2::new(width * cell, height * cell),
            ),
            0.0,
            color,
        );
    };

    pixel(6.0, 10.0, 2.0, 2.0, Color32::from_rgb(255, 213, 171));
    pixel(9.0, 10.0, 2.0, 2.0, Color32::from_rgb(255, 213, 171));

    let berry_x = berry_x + (approach * 2.0).round() * 0.15;
    pixel(
        berry_x,
        berry_y + 0.5,
        2.0,
        2.0,
        Color32::from_rgb(178, 44, 71),
    );
    pixel(
        berry_x + 0.5,
        berry_y,
        2.0,
        2.0,
        Color32::from_rgb(237, 74, 98),
    );
    pixel(
        berry_x + 1.0,
        berry_y + 0.5,
        1.0,
        1.0,
        Color32::from_rgb(255, 159, 152),
    );
    pixel(berry_x + 0.5, berry_y - 0.5, 1.0, 0.5, MINT);
    pixel(berry_x + 1.0, berry_y - 0.5, 1.0, 0.5, MINT);

    let chewing = phase > 0.72;
    if chewing && (time * 13.0).sin() > 0.0 {
        pixel(7.0, 11.0, 2.0, 2.0, INK);
        pixel(7.0, 12.0, 1.0, 1.0, Color32::from_rgb(244, 117, 145));
    } else {
        pixel(7.0, 11.0, 2.0, 1.0, INK);
    }

    if phase > 0.84 {
        let sparkle_alpha = ((1.0 - phase) * 1500.0).clamp(0.0, 220.0) as u8;
        for (x, y) in [(5.0, 9.0), (11.0, 10.0), (8.0, 8.5)] {
            painter.text(
                Pos2::new(origin.x + x * cell, origin.y + y * cell),
                egui::Align2::CENTER_CENTER,
                "+",
                egui::FontId::monospace(cell * 1.25),
                with_alpha(LEMON, sparkle_alpha),
            );
        }
    }
}

fn paint_sleeping_pet(painter: &egui::Painter, origin: Pos2, cell: f32, pet: PetKind, time: f64) {
    let (fur, cream, dark) = pet.colors();
    let breath = (time as f32 * 2.4).sin() * 0.16;
    let pixel = |x: f32, y: f32, width: f32, height: f32, color: Color32| {
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(origin.x + x * cell, origin.y + y * cell),
                Vec2::new(width * cell, height * cell),
            ),
            0.0,
            color,
        );
    };

    let pillow = Rect::from_min_size(
        Pos2::new(origin.x + cell, origin.y + cell * 12.0),
        Vec2::new(cell * 14.0, cell * 2.0),
    );
    painter.rect_filled(
        pillow.translate(Vec2::new(0.0, cell * 0.35)),
        0.0,
        Color32::from_rgba_unmultiplied(15, 16, 22, 75),
    );
    pixel(1.0, 12.0, 14.0, 2.0, Color32::from_rgb(122, 147, 177));
    pixel(2.0, 11.0, 12.0, 2.0, Color32::from_rgb(190, 207, 229));
    pixel(3.0, 11.0, 10.0, 1.0, Color32::from_rgb(222, 232, 245));

    pixel(1.0, 7.0, 4.0, 3.0, dark);
    pixel(1.0, 7.0, 3.0, 2.0, fur);
    pixel(3.0, 6.0, 10.0, 6.0 + breath, dark);
    pixel(4.0, 6.0 + breath, 8.0, 5.0, fur);
    pixel(5.0, 7.0 + breath, 6.0, 3.0, cream);
    pixel(4.0, 10.0, 2.0, 2.0, fur);
    pixel(10.0, 10.0, 2.0, 2.0, fur);

    pixel(9.0, 3.0, 7.0, 7.0, dark);
    pixel(10.0, 3.0, 5.0, 6.0, fur);
    pixel(10.0, 1.0, 2.0, 3.0, dark);
    pixel(11.0, 2.0, 1.0, 2.0, fur);
    pixel(14.0, 1.0, 2.0, 3.0, dark);
    pixel(14.0, 2.0, 1.0, 2.0, fur);
    pixel(10.0, 6.0, 5.0, 3.0, cream);
    pixel(11.0, 5.0, 1.0, 0.5, INK);
    pixel(14.0, 5.0, 1.0, 0.5, INK);
    pixel(12.0, 7.0, 1.0, 0.7, dark);
    pixel(11.0, 8.0, 1.0, 0.5, dark);
    pixel(13.0, 8.0, 1.0, 0.5, dark);

    let z_offset = (time as f32 * 10.0) % (cell * 4.0);
    painter.text(
        Pos2::new(origin.x + cell * 15.0, origin.y + cell * 3.0 - z_offset),
        egui::Align2::CENTER_CENTER,
        "Z",
        egui::FontId::monospace(cell * 2.0),
        with_alpha(APP_TEXT, 210),
    );
    painter.text(
        Pos2::new(
            origin.x + cell * 13.0,
            origin.y + cell * 5.0 - z_offset * 0.65,
        ),
        egui::Align2::CENTER_CENTER,
        "z",
        egui::FontId::monospace(cell * 1.5),
        with_alpha(APP_TEXT, 170),
    );
}

fn paint_pixel_bubble(painter: &egui::Painter, anchor: Pos2, text: &str, width: f32) {
    let width = width.max(150.0).min(250.0);
    let height = 38.0;
    let rect = Rect::from_min_size(
        Pos2::new(anchor.x - width * 0.5, anchor.y - height - 8.0),
        Vec2::new(width, height),
    );
    painter.rect_filled(rect.translate(Vec2::new(3.0, 3.0)), 0.0, APP_BORDER);
    painter.rect_filled(rect, 0.0, Color32::from_rgb(255, 244, 207));
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(2.0_f32, INK),
        egui::StrokeKind::Inside,
    );
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(anchor.x - 7.0, rect.bottom() - 1.0),
            Vec2::new(14.0, 6.0),
        ),
        0.0,
        Color32::from_rgb(255, 244, 207),
    );
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(anchor.x - 3.0, rect.bottom() + 4.0),
            Vec2::new(7.0, 5.0),
        ),
        0.0,
        INK,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::monospace(11.0),
        INK,
    );
}

#[cfg(target_os = "windows")]
fn speak_pet_line(line: &str) -> Result<(), String> {
    let line = chat::strip_emojis(line);
    if line.trim().is_empty() {
        return Ok(());
    }
    let mut child = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            "Add-Type -AssemblyName System.Speech; $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; $synth.SelectVoiceByHints([System.Speech.Synthesis.VoiceGender]::Female); $line = [Console]::In.ReadToEnd(); $safe = [System.Security.SecurityElement]::Escape($line); $ssml = \"<speak version='1.0' xml:lang='en-US'><prosody pitch='+55%' rate='+12%'>$safe</prosody></speak>\"; $synth.SpeakSsml($ssml)",
        ])
        .creation_flags(0x0800_0000)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start Windows speech synthesis: {error}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Windows speech synthesis input was unavailable".to_owned())?;
    stdin
        .write_all(line.as_bytes())
        .map_err(|error| format!("could not send the voice line to speech synthesis: {error}"))?;
    drop(stdin);

    std::thread::Builder::new()
        .name("pocket-pal-voice".to_owned())
        .spawn(move || match child.wait_with_output() {
            Ok(output) if !output.status.success() => eprintln!(
                "Pocket Pal speech synthesis failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            Err(error) => eprintln!("Pocket Pal could not finish speech synthesis: {error}"),
            _ => {}
        })
        .map_err(|error| format!("could not monitor speech synthesis: {error}"))?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn speak_pet_line(_line: &str) -> Result<(), String> {
    Err("pet voice lines currently require Windows speech synthesis".to_owned())
}

fn paint_room(
    painter: &egui::Painter,
    room: Rect,
    save: &SaveData,
    time: f64,
    celebrating: bool,
    care: Option<(CareTaskKind, f32)>,
    speech: Option<&str>,
) {
    painter.rect_filled(room, 3.0, Color32::from_rgb(247, 215, 173));
    painter.rect_filled(
        Rect::from_min_max(
            room.min,
            Pos2::new(room.right(), room.top() + room.height() * 0.69),
        ),
        3.0,
        Color32::from_rgb(249, 225, 193),
    );
    let grid = 16.0;
    for x in 0..=(room.width() / grid) as usize {
        let xpos = room.left() + x as f32 * grid;
        painter.line_segment(
            [Pos2::new(xpos, room.top()), Pos2::new(xpos, room.bottom())],
            Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(205, 176, 147, 48)),
        );
    }
    let floor_y = room.top() + room.height() * 0.69;
    painter.rect_filled(
        Rect::from_min_max(Pos2::new(room.left(), floor_y), room.max),
        0.0,
        Color32::from_rgb(197, 149, 113),
    );
    for x in 0..(room.width() / 26.0) as usize {
        let x = room.left() + x as f32 * 26.0;
        painter.line_segment(
            [Pos2::new(x, floor_y), Pos2::new(x - 9.0, room.bottom())],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(145, 105, 84, 70)),
        );
    }

    let window = Rect::from_min_size(
        Pos2::new(room.left() + 28.0, room.top() + 28.0),
        Vec2::new(98.0, 78.0),
    );
    painter.rect_filled(window.expand(6.0), 0.0, Color32::from_rgb(150, 112, 93));
    painter.rect_filled(window, 0.0, Color32::from_rgb(183, 218, 223));
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(window.left() + 4.0, window.top() + 30.0),
            Vec2::new(90.0, 48.0),
        ),
        0.0,
        Color32::from_rgb(169, 205, 178),
    );
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(window.left() + 46.0, window.top()),
            Vec2::new(6.0, 78.0),
        ),
        0.0,
        Color32::from_rgb(255, 243, 211),
    );
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(window.left(), window.top() + 36.0),
            Vec2::new(98.0, 6.0),
        ),
        0.0,
        Color32::from_rgb(255, 243, 211),
    );
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(window.center().x - 37.0, window.top() + 49.0),
            Vec2::new(30.0, 4.0),
        ),
        0.0,
        Color32::from_rgb(110, 163, 128),
    );
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(window.center().x + 8.0, window.top() + 59.0),
            Vec2::new(37.0, 4.0),
        ),
        0.0,
        Color32::from_rgb(110, 163, 128),
    );
    for (offset, color) in [
        (Vec2::new(15.0, 15.0), LEMON),
        (Vec2::new(70.0, 18.0), Color32::from_rgb(255, 247, 226)),
    ] {
        let center = Pos2::new(window.left() + offset.x, window.top() + offset.y);
        painter.rect_filled(Rect::from_center_size(center, Vec2::splat(8.0)), 0.0, color);
    }

    if save.placed_decor.contains(&Decoration::Picture) {
        let frame = Rect::from_min_size(
            Pos2::new(room.right() - 116.0, room.top() + 35.0),
            Vec2::new(74.0, 56.0),
        );
        painter.rect_filled(frame.expand(5.0), 0.0, Color32::from_rgb(169, 125, 86));
        painter.rect_filled(frame, 0.0, Color32::from_rgb(175, 216, 221));
        painter.rect_filled(
            Rect::from_two_pos(
                Pos2::new(frame.left(), frame.center().y + 3.0),
                Pos2::new(frame.right(), frame.bottom()),
            ),
            0.0,
            Color32::from_rgb(117, 169, 130),
        );
        painter.add(egui::Shape::convex_polygon(
            vec![
                Pos2::new(frame.left() + 9.0, frame.center().y + 5.0),
                Pos2::new(frame.center().x - 3.0, frame.top() + 12.0),
                Pos2::new(frame.center().x + 19.0, frame.center().y + 5.0),
            ],
            Color32::from_rgb(114, 147, 129),
            Stroke::NONE,
        ));
    }

    if save.placed_decor.contains(&Decoration::Rug) {
        painter.rect_filled(
            Rect::from_center_size(
                Pos2::new(room.center().x + 12.0, floor_y + room.height() * 0.2),
                Vec2::new(room.width() * 0.49, 26.0),
            ),
            0.0,
            Color32::from_rgb(229, 164, 132),
        );
        painter.rect_filled(
            Rect::from_center_size(
                Pos2::new(room.center().x + 12.0, floor_y + room.height() * 0.2),
                Vec2::new(room.width() * 0.43, 14.0),
            ),
            0.0,
            Color32::from_rgb(246, 202, 158),
        );
    }

    if save.placed_decor.contains(&Decoration::Plant) {
        let x = window.right() + 30.0;
        let y = floor_y - 4.0;
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x - 10.0, y - 25.0), Vec2::new(20.0, 22.0)),
            0.0,
            MINT,
        );
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x - 14.0, y - 4.0), Vec2::new(28.0, 17.0)),
            0.0,
            Color32::from_rgb(186, 119, 93),
        );
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x - 6.0, y - 38.0), Vec2::new(12.0, 14.0)),
            0.0,
            Color32::from_rgb(122, 175, 119),
        );
    }

    if save.placed_decor.contains(&Decoration::Lamp) {
        let x = room.right() - 45.0;
        let y = floor_y - 10.0;
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x - 3.0, y - 52.0), Vec2::new(6.0, 48.0)),
            0.0,
            Color32::from_rgb(156, 110, 82),
        );
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x - 14.0, y - 62.0), Vec2::new(28.0, 14.0)),
            0.0,
            LEMON,
        );
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x - 10.0, y - 67.0), Vec2::new(20.0, 5.0)),
            0.0,
            MELON,
        );
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x - 12.0, y - 4.0), Vec2::new(24.0, 5.0)),
            0.0,
            Color32::from_rgb(156, 110, 82),
        );
    }

    if save.placed_decor.contains(&Decoration::Cushion) {
        let center = Pos2::new(room.left() + room.width() * 0.22, floor_y + 34.0);
        painter.rect_filled(
            Rect::from_center_size(center, Vec2::new(32.0, 22.0)),
            0.0,
            SKY,
        );
        painter.rect_filled(
            Rect::from_center_size(center, Vec2::new(18.0, 6.0)),
            0.0,
            CREAM,
        );
        painter.rect_filled(
            Rect::from_center_size(center, Vec2::new(6.0, 18.0)),
            0.0,
            CREAM,
        );
    }

    let pet = save.selected_pet;
    let cell = (room.width().min(room.height()) / 39.0).clamp(6.0, 10.0);
    let sprite_width = 16.0 * cell;
    let sprite_height = 16.0 * cell;
    let bounce = ((time * 2.7).sin() * 3.0) as f32;
    let origin = Pos2::new(
        room.center().x - sprite_width * 0.5,
        floor_y - sprite_height + bounce,
    );
    let needs = save.needs.get(&pet).copied().unwrap_or_default();
    paint_pet_sprite(
        painter,
        origin,
        cell,
        pet,
        pet_mood_state_for(needs, celebrating),
        care,
        time,
    );
    if let Some(line) = speech {
        paint_pixel_bubble(
            painter,
            Pos2::new(room.center().x, origin.y - 5.0),
            line,
            room.width().min(250.0),
        );
    }
    if save.placed_decor.contains(&Decoration::Cushion) {
        painter.text(
            Pos2::new(room.center().x, floor_y + 8.0),
            egui::Align2::CENTER_CENTER,
            "+",
            egui::FontId::proportional(17.0),
            ROSE,
        );
    }
    painter.text(
        Pos2::new(room.center().x, room.bottom() - 14.0),
        egui::Align2::CENTER_CENTER,
        format!("{}'s little home", pet.name()),
        egui::FontId::proportional(11.0),
        Color32::from_rgb(255, 245, 220),
    );
}

fn stat_bar(ui: &mut egui::Ui, label: &str, value: u8, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(9.0)
                .strong()
                .color(APP_MUTED),
        );
        ui.add_space(5.0);
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width() - 33.0, 11.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(3), APP_SURFACE_HI);
        let filled = Rect::from_min_size(
            rect.min,
            Vec2::new(rect.width() * value as f32 / 100.0, rect.height()),
        );
        ui.painter()
            .rect_filled(filled, egui::CornerRadius::same(3), color);
        ui.label(
            egui::RichText::new(format!("{value}%"))
                .size(9.0)
                .color(APP_MUTED),
        );
    });
    ui.add_space(5.0);
}

fn pet_mood_state(needs: PetNeeds) -> PetMood {
    if needs.energy < 30 {
        PetMood::Sleepy
    } else if needs.fullness < 35 {
        PetMood::Hungry
    } else if needs.joy < 30 {
        PetMood::Sad
    } else {
        PetMood::Happy
    }
}

fn pet_mood_state_for(needs: PetNeeds, celebrating: bool) -> PetMood {
    if celebrating {
        PetMood::Happy
    } else {
        pet_mood_state(needs)
    }
}

fn pet_mood(needs: PetNeeds, celebrating: bool) -> &'static str {
    match pet_mood_state_for(needs, celebrating) {
        PetMood::Happy if needs.joy > 75 && needs.fullness > 55 => {
            "Feeling like the luckiest little pal"
        }
        PetMood::Happy => "Happy to spend a little time together",
        PetMood::Sleepy => "A little sleepy. A nap might be nice",
        PetMood::Hungry => "Could go for a tiny snack",
        PetMood::Sad => "Could use a little play time",
    }
}

fn chunky_button(ui: &mut egui::Ui, text: &str, _color: Color32) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .size(11.0)
                .strong()
                .color(APP_TEXT),
        )
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .corner_radius(2.0)
        .min_size(Vec2::new(88.0, 26.0)),
    )
}

fn small_button(ui: &mut egui::Ui, text: &str, _color: Color32) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(text).size(9.0).strong().color(APP_TEXT))
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
            .corner_radius(2.0)
            .min_size(Vec2::new(46.0, 21.0)),
    )
}

fn floating_small_button(ui: &mut egui::Ui, text: &str, color: Color32) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(text).size(9.0).strong().color(APP_TEXT))
            .fill(color)
            .stroke(Stroke::new(1.0_f32, APP_BORDER))
            .corner_radius(7.0)
            .min_size(Vec2::new(54.0, 25.0)),
    )
}

fn paint_floating_action_icon(
    painter: &egui::Painter,
    center: Pos2,
    action: usize,
    color: Color32,
) {
    let stroke = Stroke::new(2.2_f32, color);
    let line = |points: &[Pos2]| {
        painter.add(egui::Shape::line(points.to_vec(), stroke));
    };
    match action {
        0 => {
            painter.circle_filled(center + Vec2::new(0.0, 3.0), 6.0, color);
            line(&[
                center + Vec2::new(-1.0, -3.0),
                center + Vec2::new(-4.0, -7.0),
            ]);
            line(&[center + Vec2::new(1.0, -3.0), center + Vec2::new(5.0, -6.0)]);
        }
        1 => {
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "Zzz",
                egui::FontId::monospace(11.0),
                color,
            );
        }
        2 => {
            painter.add(egui::Shape::convex_polygon(
                vec![
                    center + Vec2::new(-5.0, -7.0),
                    center + Vec2::new(7.0, 0.0),
                    center + Vec2::new(-5.0, 7.0),
                ],
                color,
                Stroke::NONE,
            ));
        }
        3 => {
            painter.rect_stroke(
                Rect::from_center_size(center + Vec2::new(0.0, -1.0), Vec2::new(15.0, 11.0)),
                2.0,
                stroke,
                egui::StrokeKind::Middle,
            );
            line(&[
                center + Vec2::new(-4.0, 4.0),
                center + Vec2::new(-6.0, 8.0),
                center + Vec2::new(1.0, 4.0),
            ]);
            painter.circle_filled(center + Vec2::new(-4.0, -1.0), 0.8, color);
            painter.circle_filled(center, 0.8, color);
            painter.circle_filled(center + Vec2::new(4.0, -1.0), 0.8, color);
        }
        4 => {
            line(&[center + Vec2::new(-5.0, -5.0), center + Vec2::new(5.0, 5.0)]);
            line(&[center + Vec2::new(5.0, -5.0), center + Vec2::new(-5.0, 5.0)]);
        }
        _ => unreachable!(),
    }
}

fn tab_button(ui: &mut egui::Ui, text: &str, selected: bool) -> egui::Response {
    let foreground = if selected { APP_TEXT } else { APP_MUTED };
    let response = ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .size(10.0)
                .strong()
                .color(foreground),
        )
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .corner_radius(2.0)
        .min_size(Vec2::new(58.0, 23.0)),
    );
    if selected {
        ui.painter().line_segment(
            [
                response.rect.left_bottom() + Vec2::new(5.0, -1.0),
                response.rect.right_bottom() + Vec2::new(-5.0, -1.0),
            ],
            Stroke::new(1.0_f32, APP_TEXT),
        );
    }
    response
}

fn draw_pet_icon(ui: &mut egui::Ui, pet: PetKind) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(38.0), Sense::hover());
    ui.painter().circle_filled(
        rect.center(),
        rect.width() * 0.5,
        Color32::from_rgb(248, 233, 208),
    );
    let cell = 2.0;
    let origin = Pos2::new(rect.center().x - 16.0, rect.center().y - 16.0);
    let (fur, cream, dark) = pet.colors();
    for (row, line) in pet.sprite().iter().enumerate() {
        for (column, pixel) in line.chars().enumerate() {
            let color = match pixel {
                'F' => fur,
                'D' => dark,
                'C' => cream,
                'E' => INK,
                'P' => ROSE,
                _ => continue,
            };
            ui.painter().rect_filled(
                Rect::from_min_size(
                    Pos2::new(
                        origin.x + column as f32 * cell,
                        origin.y + row as f32 * cell,
                    ),
                    Vec2::splat(cell),
                ),
                0.0,
                color,
            );
        }
    }
}

fn round_icon_badge(ui: &mut egui::Ui, glyph: &str, color: Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(9), color);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::proportional(size * 0.48),
        APP_TEXT,
    );
}

fn decoration_glyph(item: Decoration) -> &'static str {
    match item {
        Decoration::Plant => "P",
        Decoration::Rug => "R",
        Decoration::Lamp => "L",
        Decoration::Picture => "PIC",
        Decoration::Cushion => "*",
    }
}

fn save_path() -> Result<std::path::PathBuf, String> {
    let base = std::env::var_os("APPDATA")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(std::path::PathBuf::from))
        .ok_or_else(|| "could not find a user data directory".to_owned())?;
    Ok(base.join("PocketPal").join("save.json"))
}

fn load_save() -> Result<Option<SaveData>, String> {
    let path = save_path()?;
    match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents)
            .map(Some)
            .map_err(|error| format!("invalid save file: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("could not read save file: {error}")),
    }
}

fn store_save(save: &SaveData) -> Result<(), String> {
    let path = save_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| "save path has no parent directory".to_owned())?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("could not create save folder: {error}"))?;
    let contents = serde_json::to_string_pretty(save)
        .map_err(|error| format!("could not encode save: {error}"))?;
    std::fs::write(path, contents).map_err(|error| format!("could not write save: {error}"))
}

#[cfg(target_os = "windows")]
fn ensure_desktop_shortcut() -> Result<(), String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("could not locate the running executable: {error}"))?;
    run_shortcut_script("ensure", "DesktopDirectory", &executable)
}

#[cfg(target_os = "windows")]
fn set_run_at_startup(enabled: bool) -> Result<(), String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("could not locate the running executable: {error}"))?;
    run_shortcut_script(
        if enabled { "ensure" } else { "remove" },
        "Startup",
        &executable,
    )
}

#[cfg(target_os = "windows")]
fn run_shortcut_script(
    action: &str,
    folder: &str,
    executable: &std::path::Path,
) -> Result<(), String> {
    let payload = serde_json::json!({ "target": executable.to_string_lossy() }).to_string();
    let script = "$ErrorActionPreference = 'Stop'; $payload = [Console]::In.ReadToEnd() | ConvertFrom-Json; $kind = [Enum]::Parse([Environment+SpecialFolder], $env:POCKET_PAL_SHORTCUT_FOLDER); $directory = [Environment]::GetFolderPath($kind); if ([string]::IsNullOrWhiteSpace($directory)) { throw 'Windows did not provide the requested folder path.' }; $link = Join-Path $directory 'Pocket Pal.lnk'; if ($env:POCKET_PAL_SHORTCUT_ACTION -eq 'remove') { if (Test-Path -LiteralPath $link) { Remove-Item -LiteralPath $link -Force }; exit 0 }; if (-not (Test-Path -LiteralPath $link)) { $shell = New-Object -ComObject WScript.Shell; $shortcut = $shell.CreateShortcut($link); $shortcut.TargetPath = $payload.target; $shortcut.WorkingDirectory = [IO.Path]::GetDirectoryName($payload.target); $shortcut.IconLocation = $payload.target + ',0'; $shortcut.Save() }";
    let mut child = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            script,
        ])
        .creation_flags(0x0800_0000)
        .env("POCKET_PAL_SHORTCUT_ACTION", action)
        .env("POCKET_PAL_SHORTCUT_FOLDER", folder)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start Windows shortcut setup: {error}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Windows shortcut setup input was unavailable".to_owned())?;
    stdin
        .write_all(payload.as_bytes())
        .map_err(|error| format!("could not send shortcut settings to Windows: {error}"))?;
    drop(stdin);
    let output = child
        .wait_with_output()
        .map_err(|error| format!("could not finish Windows shortcut setup: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Windows shortcut setup failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(target_os = "windows")]
fn create_tray(
    ctx: egui::Context,
    overlay_state: Arc<Mutex<FloatingOverlayState>>,
) -> Result<TrayIntegration, String> {
    let restore_item = MenuItem::new("Show Pocket Pal", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let menu = Menu::new();
    menu.append_items(&[&restore_item, &quit_item])
        .map_err(|error| format!("could not build tray menu: {error}"))?;

    let mut rgba = vec![0_u8; 32 * 32 * 4];
    for y in 0..32 {
        for x in 0..32 {
            let in_ear_left = (8..14).contains(&x) && (4..12).contains(&y) && x + y < 22;
            let in_ear_right = (18..24).contains(&x) && (4..12).contains(&y) && y < x - 5;
            let in_face = (6..26).contains(&x)
                && (10..26).contains(&y)
                && (x >= 9 || y >= 13)
                && (x <= 22 || y >= 13);
            let color = if (10..13).contains(&x) && (16..19).contains(&y)
                || (19..22).contains(&x) && (16..19).contains(&y)
                || (14..18).contains(&x) && (21..23).contains(&y)
            {
                Some([50, 42, 52, 255])
            } else if in_ear_left || in_ear_right || in_face {
                Some([204, 139, 102, 255])
            } else {
                None
            };
            if let Some(color) = color {
                let offset = (y * 32 + x) * 4;
                rgba[offset..offset + 4].copy_from_slice(&color);
            }
        }
    }

    let icon = tray_icon::Icon::from_rgba(rgba, 32, 32)
        .map_err(|error| format!("could not create tray icon image: {error}"))?;
    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("Pocket Pal")
        .with_icon(icon)
        .with_menu_on_left_click(false)
        .with_menu(Box::new(menu))
        .build()
        .map_err(|error| format!("could not register tray icon: {error}"))?;

    let restore_id = restore_item.id().clone();
    let quit_id = quit_item.id().clone();
    let menu_ctx = ctx.clone();
    let menu_overlay = Arc::clone(&overlay_state);
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if event.id == restore_id {
            request_tray_restore(&menu_ctx, &menu_overlay);
        } else if event.id == quit_id {
            menu_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            menu_ctx.request_repaint();
        }
    }));

    let tray_id = tray_icon.id().clone();
    let tray_ctx = ctx;
    let tray_overlay = overlay_state;
    TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
        if event.id() == &tray_id
            && matches!(
                event,
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } | TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            )
        {
            request_tray_restore(&tray_ctx, &tray_overlay);
        }
    }));

    Ok(TrayIntegration { _icon: tray_icon })
}

#[cfg(target_os = "windows")]
fn request_tray_restore(ctx: &egui::Context, overlay: &Arc<Mutex<FloatingOverlayState>>) {
    if let Ok(mut state) = overlay.lock() {
        state.actions_open = true;
        state.restore_requested = true;
    } else {
        eprintln!(
            "Pocket Pal could not reveal floating controls because their state lock was poisoned"
        );
    }
    show_root_viewport(ctx);
    ctx.request_repaint();
    ctx.request_repaint_after(Duration::from_millis(100));
}

fn show_root_viewport(ctx: &egui::Context) {
    ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Visible(true));
    ctx.send_viewport_cmd_to(
        egui::ViewportId::ROOT,
        egui::ViewportCommand::Minimized(false),
    );
    ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Focus);
}

fn request_floating_exit(
    floating_ctx: &egui::Context,
    root_ctx: &egui::Context,
    overlay: &Arc<Mutex<FloatingOverlayState>>,
) {
    match overlay.lock() {
        Ok(mut state) => {
            state.actions_open = false;
            state.exit_requested = true;
            match state.chat.lock() {
                Ok(mut chat) => chat.open = false,
                Err(_) => eprintln!(
                    "Pocket Pal could not close floating chat because its state lock was poisoned"
                ),
            }
        }
        Err(_) => {
            eprintln!(
                "Pocket Pal could not exit floating mode because its state lock was poisoned"
            );
            return;
        }
    }

    if !floating_ctx.embed_viewports() {
        root_ctx.send_viewport_cmd_to(floating_pet_viewport_id(), egui::ViewportCommand::Close);
    }
    root_ctx.send_viewport_cmd_to(
        egui::ViewportId::ROOT,
        egui::ViewportCommand::Minimized(false),
    );
    root_ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Visible(true));
    root_ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Focus);
    root_ctx.request_repaint();
}

fn floating_pet_viewport_id() -> egui::ViewportId {
    egui::ViewportId::from_hash_of("pocket_pal_floating_pet")
}

fn advance_floating_overlay(state: &mut FloatingOverlayState, now: f64) {
    let dt = state
        .last_update
        .replace(now)
        .map_or(0.0, |previous| (now - previous).clamp(0.0, 0.1) as f32);
    if !state.dirty || dt <= 0.0 {
        return;
    }

    state.celebration_time = (state.celebration_time - dt).max(0.0);
    state.celebrating = state.celebration_time > 0.0;
    if let Some((kind, progress)) = &mut state.care_task {
        let duration = match kind {
            CareTaskKind::Snack => 5.0,
            CareTaskKind::Nap => 7.0,
        };
        *progress = (*progress + dt / duration).min(1.0);
        if *progress >= 1.0 {
            match kind {
                CareTaskKind::Snack => {
                    state.needs.fullness = state.needs.fullness.saturating_add(24).min(100)
                }
                CareTaskKind::Nap => {
                    state.needs.energy = state.needs.energy.saturating_add(28).min(100)
                }
            }
            state.needs.joy = state.needs.joy.saturating_add(35).min(100);
            state.care_task = None;
            state.celebrating = true;
            state.celebration_time = 3.0;
        }
    }

    if let Some((_, time_left, target)) = &mut state.game {
        *time_left = (*time_left - dt).max(0.0);
        state.game_move_timer = (state.game_move_timer - dt).max(0.0);
        if state.game_move_timer <= 0.0 {
            let mut game = StarGame::new();
            game.target = *target;
            game.target_step = state.game_target_step;
            game.move_target();
            *target = game.target;
            state.game_target_step = game.target_step;
            state.game_move_timer = 1.1;
        }
        if *time_left <= 0.0 {
            state.game = None;
        }
    }
}

fn show_floating_overlay(
    ctx: &egui::Context,
    state: &Arc<Mutex<FloatingOverlayState>>,
    root_ctx: &egui::Context,
) {
    ctx.request_repaint_after(Duration::from_millis(32));
    let snapshot = match state.lock() {
        Ok(mut state) => {
            advance_floating_overlay(&mut state, ctx.input(|input| input.time));
            state.clone()
        }
        Err(_) => {
            eprintln!(
                "Pocket Pal could not render the floating pet because its state lock was poisoned"
            );
            return;
        }
    };

    if let Some((score, time_left, target)) = snapshot.game {
        egui::Area::new(egui::Id::new("floating_star_game"))
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                let (screen, _) = ui.allocate_exact_size(Vec2::new(256.0, 256.0), Sense::hover());
                let painter = ui.painter();
                for index in 0..18 {
                    let x = ((index * 197 + 41) % 251) as f32;
                    let y = ((index * 113 + 29) % 251) as f32;
                    let radius = if index % 4 == 0 { 5.0 } else { 2.5 };
                    let center = Pos2::new(screen.left() + x, screen.top() + y);
                    painter.add(egui::Shape::convex_polygon(
                        star_points(center, radius, radius * 0.45),
                        with_alpha(LEMON, if index % 4 == 0 { 150 } else { 85 }),
                        Stroke::NONE,
                    ));
                }

                let pet_origin = Pos2::new(screen.center().x - 48.0, screen.top() + 88.0);
                paint_pet_sprite(
                    painter,
                    pet_origin,
                    6.0,
                    snapshot.pet,
                    pet_mood_state_for(snapshot.needs, snapshot.celebrating),
                    snapshot.care_task,
                    ctx.input(|input| input.time),
                );
                painter.text(
                    Pos2::new(screen.center().x, screen.top() + 12.0),
                    egui::Align2::CENTER_TOP,
                    format!("{score} / 5  ·  {time_left:.0}s"),
                    egui::FontId::proportional(12.0),
                    APP_TEXT,
                );

                let star_center = Pos2::new(
                    screen.left() + 22.0 + target.0 * (screen.width() - 44.0).max(1.0),
                    screen.top() + 22.0 + target.1 * (screen.height() - 44.0).max(1.0),
                );
                let target_response = ui.interact(
                    Rect::from_center_size(star_center, Vec2::splat(46.0)),
                    egui::Id::new("floating_star_target"),
                    Sense::click(),
                );
                let star_radius = if target_response.hovered() {
                    17.0
                } else {
                    14.0
                };
                painter.add(egui::Shape::convex_polygon(
                    star_points(star_center, star_radius, star_radius * 0.46),
                    LEMON,
                    Stroke::new(1.0_f32, Color32::from_rgb(255, 249, 218)),
                ));
                if target_response.clicked() {
                    if let Ok(mut state) = state.lock() {
                        let target_step = state.game_target_step;
                        if let Some((score, _, target)) = &mut state.game {
                            *score = score.saturating_add(1);
                            if *score >= 5 {
                                state.game = None;
                                state.needs.joy = state.needs.joy.saturating_add(24).min(100);
                                state.needs.energy = state.needs.energy.saturating_sub(8);
                                state.reward_pending = true;
                                state.celebrating = true;
                                state.celebration_time = 3.0;
                            } else {
                                let mut game = StarGame::new();
                                game.target = *target;
                                game.target_step = target_step;
                                game.move_target();
                                *target = game.target;
                                state.game_target_step = game.target_step;
                                state.game_move_timer = 0.45;
                            }
                            state.dirty = true;
                        }
                    }
                    root_ctx.request_repaint();
                }

                let exit_rect =
                    Rect::from_min_size(Pos2::new(screen.right() - 30.0, 4.0), Vec2::splat(24.0));
                let exit_button = ui.interact(
                    exit_rect,
                    egui::Id::new("floating_star_exit"),
                    Sense::click(),
                );
                paint_floating_action_icon(
                    ui.painter(),
                    exit_rect.center(),
                    4,
                    if exit_button.hovered() {
                        APP_ACCENT
                    } else {
                        APP_TEXT
                    },
                );
                if exit_button.clicked() {
                    if let Ok(mut state) = state.lock() {
                        state.game = None;
                        state.dirty = true;
                    }
                    root_ctx.request_repaint();
                }
            });
        show_floating_chat(ctx, Arc::clone(&snapshot.chat), snapshot.pet);
        return;
    }

    let progress = ctx.animate_bool_with_time(
        egui::Id::new("floating_pet_actions"),
        snapshot.actions_open,
        0.24,
    );
    egui::Area::new(egui::Id::new("floating_pet_card"))
        .order(egui::Order::Foreground)
        .fixed_pos(Pos2::ZERO)
        .show(ctx, |ui| {
            let (canvas, _) = ui.allocate_exact_size(Vec2::new(256.0, 256.0), Sense::hover());
            let bounce = ((ctx.input(|input| input.time) * 2.7).sin() * 1.5) as f32;
            let pet_origin = Pos2::new(canvas.left() + 64.0, canvas.top() + 64.0 + bounce);
            let response = ui.interact(
                Rect::from_min_size(pet_origin, Vec2::splat(128.0)),
                egui::Id::new("floating_pet_click"),
                Sense::click_and_drag(),
            );
            if response.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if response.clicked() {
                match state.lock() {
                    Ok(mut overlay) => overlay.actions_open = !overlay.actions_open,
                    Err(_) => eprintln!("Pocket Pal could not toggle floating controls because their state lock was poisoned"),
                }
                root_ctx.request_repaint();
            }
            paint_pet_sprite(
                ui.painter(),
                pet_origin,
                8.0,
                snapshot.pet,
                pet_mood_state_for(snapshot.needs, snapshot.celebrating),
                snapshot.care_task,
                ctx.input(|input| input.time),
            );
            if let Some(line) = snapshot.voice_line.as_deref() {
                paint_pixel_bubble(
                    ui.painter(),
                    Pos2::new(canvas.center().x, pet_origin.y - 5.0),
                    line,
                    226.0,
                );
            }

            if let Some((kind, task_progress)) = snapshot.care_task {
                let task_color = match kind {
                    CareTaskKind::Snack => MELON,
                    CareTaskKind::Nap => SKY,
                };
                let line = Rect::from_min_max(
                    Pos2::new(canvas.center().x - 44.0, canvas.bottom() - 13.0),
                    Pos2::new(canvas.center().x + 44.0, canvas.bottom() - 11.0),
                );
                ui.painter().line_segment(
                    [line.left_center(), line.right_center()],
                    Stroke::new(1.0_f32, with_alpha(APP_SURFACE_HI, 180)),
                );
                ui.painter().line_segment(
                    [
                        line.left_center(),
                        Pos2::new(
                            line.left() + line.width() * task_progress.clamp(0.0, 1.0),
                            line.center().y,
                        ),
                    ],
                    Stroke::new(2.0_f32, with_alpha(task_color, 230)),
                );
            }

            if snapshot.actions_open || progress > 0.0 {
                let alpha = (progress * 255.0) as u8;
                let actions = [
                    (FLOAT_FEED, Vec2::new(128.0, 19.0)),
                    (FLOAT_NAP, Vec2::new(224.0, 90.0)),
                    (FLOAT_PLAY, Vec2::new(224.0, 218.0)),
                    (FLOAT_CHAT, Vec2::new(32.0, 218.0)),
                    (Color32::from_gray(65), Vec2::new(32.0, 90.0)),
                ];
                for (index, (color, center)) in actions.into_iter().enumerate() {
                    let direction = (center - Vec2::splat(128.0)).normalized();
                    let button_center =
                        canvas.min + center + direction * ((1.0 - progress) * 10.0);
                    let rect = Rect::from_center_size(button_center, Vec2::splat(40.0));
                    let button = ui.interact(
                        rect,
                        egui::Id::new(("floating_pet_action", index)),
                        Sense::click(),
                    );
                    let button_color = if button.is_pointer_button_down_on() {
                        color.gamma_multiply(0.78)
                    } else if button.hovered() {
                        color.gamma_multiply(1.12)
                    } else {
                        color
                    };
                    ui.painter().rect_filled(
                        rect,
                        egui::CornerRadius::same(12),
                        with_alpha(button_color, alpha),
                    );
                    ui.painter().rect_stroke(
                        rect,
                        egui::CornerRadius::same(12),
                        Stroke::new(1.0_f32, with_alpha(APP_BORDER, alpha)),
                        egui::StrokeKind::Middle,
                    );
                    paint_floating_action_icon(
                        ui.painter(),
                        button_center,
                        index,
                        with_alpha(APP_TEXT, alpha),
                    );
                    if button.clicked() {
                        if let Ok(mut overlay) = state.lock() {
                            overlay.actions_open = false;
                        }
                        match index {
                            0 | 1 => {
                                if let Ok(mut state) = state.lock() {
                                    if state.care_task.is_none() {
                                        state.care_task = Some((
                                            if index == 0 {
                                                CareTaskKind::Snack
                                            } else {
                                                CareTaskKind::Nap
                                            },
                                            0.0,
                                        ));
                                        state.dirty = true;
                                    }
                                }
                            }
                            2 => {
                                if let Ok(mut state) = state.lock() {
                                    state.game = Some((0, 15.0, (0.5, 0.5)));
                                    state.game_move_timer = 1.1;
                                    state.game_target_step = 0;
                                    state.dirty = true;
                                }
                            }
                            3 => {
                                let chat_state = match state.lock() {
                                    Ok(mut overlay) => {
                                        overlay.actions_open = false;
                                        Arc::clone(&overlay.chat)
                                    }
                                    Err(_) => {
                                        eprintln!("Pocket Pal could not open chat because its floating state lock was poisoned");
                                        return;
                                    }
                                };
                                toggle_floating_chat(ctx, &chat_state);
                                root_ctx.request_repaint();
                            }
                            4 => request_floating_exit(ctx, root_ctx, state),
                            _ => unreachable!(),
                        }
                    }
                }
            }
        });
    show_floating_chat(ctx, Arc::clone(&snapshot.chat), snapshot.pet);
}

fn toggle_floating_chat(ctx: &egui::Context, chat: &Arc<Mutex<FloatingChatState>>) {
    match chat.lock() {
        Ok(mut chat) => {
            chat.open = !chat.open;
            if !ctx.embed_viewports() {
                let size = if chat.open {
                    Vec2::new(380.0, 640.0)
                } else {
                    Vec2::new(256.0, 256.0)
                };
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            }
        }
        Err(_) => eprintln!(
            "Pocket Pal could not toggle floating chat because its state lock was poisoned"
        ),
    }
}

fn show_floating_chat(ctx: &egui::Context, chat: Arc<Mutex<FloatingChatState>>, pet: PetKind) {
    let mut request = None;
    let mut speak = None;
    let mut close = false;
    {
        let mut state = match chat.lock() {
            Ok(state) => state,
            Err(_) => {
                eprintln!(
                    "Pocket Pal could not render floating chat because its state lock was poisoned"
                );
                return;
            }
        };
        if !state.open {
            return;
        }
        if state.checked_provider != Some(state.provider) {
            state.checked_provider = Some(state.provider);
            state.saved_key = match chat::has_saved_key(state.provider) {
                Ok(saved) => saved,
                Err(error) => {
                    state.error = Some(error);
                    false
                }
            };
        }
        if let Some(receiver) = &state.reply_receiver {
            match receiver.try_recv() {
                Ok(Ok(reply)) => {
                    if state.voice_enabled {
                        speak = Some(reply.clone());
                    }
                    state.messages.push(chat::Message {
                        speaker: chat::Speaker::Pet,
                        text: reply,
                    });
                    state.pending = false;
                    state.error = None;
                    state.reply_receiver = None;
                }
                Ok(Err(error)) => {
                    state.pending = false;
                    state.error = Some(error);
                    state.reply_receiver = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    state.pending = false;
                    state.error =
                        Some("The chat request ended without a response. Please try again.".into());
                    state.reply_receiver = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if state.pending {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        egui::Area::new(egui::Id::new("floating_chat_panel"))
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::new(8.0, 262.0))
            .show(ctx, |ui| {
                egui::Frame::default()
                    .fill(APP_PANEL)
                    .stroke(Stroke::new(1.0_f32, APP_BORDER))
                    .corner_radius(egui::CornerRadius::same(14))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(344.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("CHAT WITH {}", pet.name()))
                                    .strong()
                                    .color(APP_TEXT),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if floating_small_button(ui, "Close", APP_SURFACE_HI).clicked()
                                    {
                                        close = true;
                                    }
                                },
                            );
                        });
                        ui.add_space(4.0);
                        let previous_provider = state.provider;
                        egui::ComboBox::from_id_salt("floating_chat_provider")
                            .selected_text(state.provider.name())
                            .show_ui(ui, |ui| {
                                for provider in chat::Provider::ALL {
                                    ui.selectable_value(
                                        &mut state.provider,
                                        provider,
                                        provider.name(),
                                    );
                                }
                            });
                        if state.provider != previous_provider {
                            state.checked_provider = None;
                        }
                        if state.checked_provider != Some(state.provider) {
                            state.checked_provider = Some(state.provider);
                            state.saved_key = match chat::has_saved_key(state.provider) {
                                Ok(saved) => saved,
                                Err(error) => {
                                    state.error = Some(error);
                                    false
                                }
                            };
                        }
                        if state.saved_key {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("API key saved securely")
                                        .size(10.0)
                                        .color(APP_MUTED),
                                );
                                if floating_small_button(ui, "Remove", APP_SURFACE_HI).clicked() {
                                    match chat::delete_key(state.provider) {
                                        Ok(()) => {
                                            state.saved_key = false;
                                            state.error = None;
                                        }
                                        Err(error) => state.error = Some(error),
                                    }
                                }
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut state.key_input)
                                        .password(true)
                                        .hint_text("Paste API key")
                                        .desired_width(220.0),
                                );
                                if floating_small_button(ui, "Save key", APP_ACCENT).clicked() {
                                    let key = state.key_input.trim();
                                    if key.is_empty() {
                                        state.error = Some("Enter an API key first.".into());
                                    } else {
                                        match chat::save_key(state.provider, key) {
                                            Ok(()) => {
                                                state.key_input.clear();
                                                state.saved_key = true;
                                                state.error = None;
                                            }
                                            Err(error) => state.error = Some(error),
                                        }
                                    }
                                }
                            });
                        }
                        #[cfg(target_os = "windows")]
                        ui.checkbox(&mut state.voice_enabled, "Speak pet replies");
                        #[cfg(not(target_os = "windows"))]
                        ui.label(
                            egui::RichText::new("Spoken replies are available on Windows.")
                                .size(10.0)
                                .color(APP_MUTED),
                        );

                        egui::ScrollArea::vertical()
                            .id_salt("floating_chat_history")
                            .max_height(176.0)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for message in &state.messages {
                                    let (speaker, fill, alignment) = match message.speaker {
                                        chat::Speaker::User => {
                                            ("YOU", APP_SURFACE_HI, egui::Align::Max)
                                        }
                                        chat::Speaker::Pet => {
                                            (pet.name(), APP_SURFACE, egui::Align::Min)
                                        }
                                    };
                                    ui.with_layout(egui::Layout::top_down(alignment), |ui| {
                                        egui::Frame::default()
                                            .fill(fill)
                                            .corner_radius(egui::CornerRadius::same(8))
                                            .inner_margin(egui::Margin::symmetric(8, 6))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new(speaker)
                                                        .size(9.0)
                                                        .strong()
                                                        .color(APP_ACCENT),
                                                );
                                                ui.label(
                                                    egui::RichText::new(&message.text)
                                                        .size(11.0)
                                                        .color(APP_TEXT),
                                                );
                                            });
                                    });
                                    ui.add_space(4.0);
                                }
                                if state.pending {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} is thinking...",
                                            pet.name()
                                        ))
                                        .size(10.0)
                                        .color(APP_MUTED),
                                    );
                                }
                            });
                        if let Some(error) = &state.error {
                            ui.label(
                                egui::RichText::new(error)
                                    .size(10.0)
                                    .color(Color32::from_rgb(255, 135, 150)),
                            );
                        }
                        ui.horizontal(|ui| {
                            let input = ui.add_enabled(
                                state.saved_key && !state.pending,
                                egui::TextEdit::singleline(&mut state.input)
                                    .char_limit(1000)
                                    .hint_text("Say something...")
                                    .desired_width(250.0),
                            );
                            let send = floating_small_button(
                                ui,
                                if state.pending { "..." } else { "Send" },
                                APP_ACCENT,
                            );
                            if state.saved_key
                                && !state.pending
                                && (send.clicked()
                                    || (input.lost_focus()
                                        && ui.input(|input| input.key_pressed(egui::Key::Enter))))
                            {
                                let text = state.input.trim().to_owned();
                                if !text.is_empty() {
                                    state.messages.push(chat::Message {
                                        speaker: chat::Speaker::User,
                                        text,
                                    });
                                    state.input.clear();
                                    state.pending = true;
                                    state.error = None;
                                    request = Some((
                                        state.provider,
                                        state.messages.clone(),
                                        pet.name().to_owned(),
                                    ));
                                }
                            }
                        });
                    });
            });
    }

    if close {
        toggle_floating_chat(ctx, &chat);
        return;
    }
    if let Some(line) = speak {
        if let Err(error) = speak_pet_line(&line) {
            eprintln!("Pocket Pal could not speak the chat reply: {error}");
        }
    }
    if let Some((provider, history, pet_name)) = request {
        let (sender, receiver) = mpsc::channel();
        match chat.lock() {
            Ok(mut state) => state.reply_receiver = Some(receiver),
            Err(_) => {
                eprintln!(
                    "Pocket Pal could not start floating chat because its state lock was poisoned"
                );
                return;
            }
        }
        let spawn_result = std::thread::Builder::new()
            .name("pocket-pal-floating-chat".to_owned())
            .spawn(move || {
                let result = chat::load_key(provider)
                    .and_then(|key| chat::send_message(provider, &key, &pet_name, &history));
                let _ = sender.send(result);
            });
        if let Err(error) = spawn_result {
            match chat.lock() {
                Ok(mut state) => {
                    state.pending = false;
                    state.reply_receiver = None;
                    state.error = Some(format!("Could not start the chat request: {error}"));
                }
                Err(_) => eprintln!("Pocket Pal could not report the chat error because its state lock was poisoned"),
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([980.0, 680.0])
            .with_min_inner_size([760.0, 540.0])
            .with_transparent(true),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Pocket Pal",
        options,
        Box::new(|cc| Ok(Box::new(PocketPal::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_save_has_starter_pals_and_a_starter_budget() {
        let save = SaveData::default();
        assert!(save.owned_pets.contains(&PetKind::Cat));
        assert!(save.owned_pets.contains(&PetKind::Frog));
        assert!(!save.owned_pets.contains(&PetKind::Fox));
        assert!(save.coins >= PetKind::Fox.unlock_cost());
    }

    #[test]
    fn pet_sprites_have_sixteen_pixel_rows() {
        for pet in PetKind::ALL {
            for row in pet.sprite() {
                assert_eq!(row.chars().count(), 16, "{} sprite row: {row}", pet.name());
            }
        }
    }

    #[test]
    fn save_data_round_trips_with_pet_and_decor_unlocks() {
        let mut save = SaveData::default();
        save.owned_pets.insert(PetKind::Bunny);
        save.unlocked_decor.insert(Decoration::Plant);
        save.placed_decor.insert(Decoration::Plant);
        save.floating_on_top = false;
        let serialized = serde_json::to_string(&save).expect("save should serialize");
        let restored: SaveData =
            serde_json::from_str(&serialized).expect("save should deserialize");
        assert!(restored.owned_pets.contains(&PetKind::Bunny));
        assert!(restored.placed_decor.contains(&Decoration::Plant));
        assert!(!restored.floating_on_top);
    }

    #[test]
    fn older_saves_default_to_always_on_top() {
        let mut save = serde_json::to_value(SaveData::default()).expect("save should serialize");
        save.as_object_mut()
            .expect("save should be an object")
            .remove("floating_on_top");
        save.as_object_mut()
            .expect("save should be an object")
            .remove("chat_provider");
        save.as_object_mut()
            .expect("save should be an object")
            .remove("run_at_startup");
        let restored: SaveData =
            serde_json::from_value(save).expect("older save should still deserialize");
        assert!(restored.floating_on_top);
        assert_eq!(restored.chat_provider, chat::Provider::OpenAi);
        assert!(!restored.run_at_startup);
    }

    #[test]
    fn pet_expression_tracks_its_needs() {
        assert_eq!(
            pet_mood_state(PetNeeds {
                energy: 20,
                fullness: 20,
                joy: 20,
            }),
            PetMood::Sleepy
        );
        assert_eq!(
            pet_mood_state(PetNeeds {
                energy: 50,
                fullness: 20,
                joy: 20,
            }),
            PetMood::Hungry
        );
        assert_eq!(
            pet_mood_state(PetNeeds {
                energy: 50,
                fullness: 50,
                joy: 20,
            }),
            PetMood::Sad
        );
        assert_eq!(pet_mood_state(PetNeeds::default()), PetMood::Happy);
    }

    #[test]
    fn care_animations_have_a_timed_progress_value() {
        let mut snack = CareTask::new(CareTaskKind::Snack);
        assert_eq!(snack.progress(), 0.0);
        assert_eq!(snack.duration, 5.0);
        snack.remaining = snack.duration * 0.5;
        assert_eq!(snack.progress(), 0.5);

        let nap = CareTask::new(CareTaskKind::Nap);
        assert_eq!(nap.duration, 7.0);
    }

    #[test]
    fn floating_care_finishes_and_updates_pet_needs() {
        let mut state = FloatingOverlayState {
            pet: PetKind::Cat,
            needs: PetNeeds::default(),
            care_task: Some((CareTaskKind::Snack, 0.0)),
            celebrating: false,
            celebration_time: 0.0,
            voice_line: None,
            game: None,
            game_move_timer: 1.1,
            game_target_step: 0,
            actions_open: false,
            dirty: true,
            reward_pending: false,
            chat: Arc::new(Mutex::new(
                FloatingChatState::new(chat::Provider::default()),
            )),
            exit_requested: false,
            restore_requested: false,
            last_update: Some(0.0),
        };

        for frame in 1..=51 {
            advance_floating_overlay(&mut state, frame as f64 * 0.1);
        }

        assert!(state.care_task.is_none());
        assert_eq!(state.needs.fullness, 96);
        assert_eq!(state.needs.joy, 100);
        assert!(state.celebrating);
    }

    #[test]
    fn star_target_moves_to_new_positions() {
        let mut game = StarGame::new();
        let first = game.target;
        game.move_target();
        let second = game.target;
        game.move_target();
        let third = game.target;

        assert_ne!(first, second);
        assert_ne!(second, third);
        for (x, y) in [first, second, third] {
            assert!((0.12..=0.88).contains(&x));
            assert!((0.12..=0.88).contains(&y));
        }
    }
}
