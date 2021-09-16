// use std::fs::File;
// use std::io::prelude::*;
use std::convert::TryFrom;
use std::{thread, time};

use serde::{Deserialize, Serialize};
//use serde_json::Result;

use enigo::*;


use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;



#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub emojis: Vec<Emoji>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Emoji {
    pub emoji: String,
    pub name: String,
    pub shortname: String,
    pub unicode: String,
    pub html: String,
    pub category: String,
    pub order: String,
}

fn parse_json_file() -> Root {
    /*
    let mut file = File::open("./src/emojis.json").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    */

    let content: &str = include_str!("./emojis.json");

    serde_json::from_str(content).unwrap()
}

use lazy_static::lazy_static;
lazy_static! {
    static ref EMOJIS: Root = parse_json_file();
}


fn find<'a>(string: &'a str) -> (Option<String>, Vec<String>) {
    if string.is_empty() {
        return (None, vec![]);
    }
    let mut helper: Vec<String> = Vec::new();

    for emoji in &EMOJIS.emojis {
        if  string.eq(&emoji.shortname) {
            println!("{:#?} vs {:#?}: {:?}", string, emoji.shortname, emoji.emoji);
            return (Some(emoji.unicode.clone()), vec![]);
        } else if emoji.shortname.contains(string.trim_matches(':')) {
            let output: String = format!("{} {}", emoji.shortname, emoji.emoji);
            helper.push(output);
        }
    }

    (None, helper)
}

fn input_unicode(unicode: &str) -> Result<(), Box<dyn std::error::Error>> {
    let hex: u32 = u32::from_str_radix(unicode, 16)?;
    let output: String = char::try_from(hex)?.into();

    let mut ctx: ClipboardContext = ClipboardProvider::new()?;
    // Fill clipboard with the emoji
    ctx.set_contents(output)?;

    //thread::sleep(time::Duration::from_millis(500));

    let mut enigo = Enigo::new();
    enigo.set_delay(100);

    enigo.key_down(Key::Alt);

    thread::sleep(time::Duration::from_millis(100));

    enigo.key_click(Key::Tab);
    enigo.key_up(Key::Alt);

    thread::sleep(time::Duration::from_millis(100));

    enigo.key_sequence_parse("{+CTRL}v{-CTRL}");
    thread::sleep(time::Duration::from_millis(100));
    enigo.key_sequence_parse("{+CTRL}{+SHIFT}v{-SHIFT}{-CTRL}");

    Ok(())
}

use druid::widget::{Align, Controller, Flex, Label, TextBox};
use druid::{AppLauncher, Data, Env, Event, EventCtx, Lens, LocalizedString, Widget, WindowDesc, WidgetExt, UnitPoint};

// const VERTICAL_WIDGET_SPACING: f64 = 20.0;
// const TEXT_BOX_WIDTH: f64 = 400.0;
const WINDOW_TITLE: LocalizedString<HelloState> = LocalizedString::new("Hello World!");

#[derive(Clone, Debug, Data, Lens)]
struct HelloState {
    name: String,
    helper: [String; 5],
}

struct UpdateCallback();

impl<W: Widget<HelloState>> Controller<HelloState, W> for UpdateCallback {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut HelloState,
        env: &Env
    ) {
        match event {
            Event::WindowConnected => {
                ctx.request_focus();
            },
            Event::KeyUp(key)  => {
                if let druid::Code::Enter = key.code {
                    //input_unicode(&data.name);
                }
                let output = find(&data.name);
                if let (None, helper) = output {
                    for i in 0..=4 {
                        data.helper[i] = helper.get(i).unwrap_or(&String::from("")).to_string();
                    }
                } else if let (Some(unicode), _) = output {
                    input_unicode(&unicode).expect("Inputting emoji failed");
                    ctx.submit_command(druid::commands::CLOSE_WINDOW);
                    data.name = "".into();
                }
            },
            _ => {}
        }

        child.event(ctx, event, data, env)
    }
    /*
    fn event(
        &mut self,
        ctx: &mut EventCtx<'_, '_>,
        event: &Event,
        data: &mut T,
        env: &Env
    ) {
        if let Event::KeyUp(_) = event {
            println!("nice");
        }
    }

    fn lifecycle(&mut self, _: &mut LifeCycleCtx<'_, '_>, _: &LifeCycle, _: &T, _: &Env) { todo!() }

    fn update(&mut self, _: &mut UpdateCtx<'_, '_>, _: &T, _: &T, _: &Env) { todo!() }

    fn layout(&mut self, _: &mut LayoutCtx<'_, '_>, _: &BoxConstraints, _: &T, _: &Env) -> druid::Size { todo!() }

    fn paint(&mut self, _: &mut PaintCtx<'_, '_, '_>, _: &T, _: &Env) { todo!() }
    */
}

fn main() {
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget)
        .title(WINDOW_TITLE)
        .with_min_size((800.0, 100.0))
        .window_size((800.0, 100.0))
        .resizable(true);

    // create the initial app state
    let initial_state = HelloState {
        name: "".into(),
        helper: Default::default(),
    };

    // start the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<HelloState> {
    // a label that will determine its text based on the current app data.
    // a textbox that modifies `name`.
    let textbox = TextBox::new()
        .with_placeholder(":emoji-name:")
        //.fix_width(TEXT_BOX_WIDTH)
        .expand_width()
        .lens(HelloState::name)
        .controller(UpdateCallback());

    // arrange the two widgets vertically, with some padding
    let mut layout = Flex::column()
        .with_child(textbox);
        //.with_spacer(VERTICAL_WIDGET_SPACING)

    let font = druid::FontDescriptor::new(druid::FontFamily::new_unchecked("Noto Sans"));
    for i in 0..=4 {
        let label = Label::new(move |data: &HelloState, _env: &Env| format!("{}", data.helper.get(i).unwrap_or(&"".into()))).with_font(font.clone());
        layout.add_child(label);
    }

    // center the two widgets in the available space
    Align::horizontal(UnitPoint::TOP, layout)
}
