mod extract;
use extract::extract_images;
use printpdf::*;
use std::convert::From;
use std::fs::File;
use std::io::BufWriter;
use chrono::prelude::*;

use fltk::{
    prelude::*,
    app::{self},
    button::Button,
    frame::Frame,
    text::{TextBuffer, TextDisplay},
    window::Window,
};

use fltk_theme::{ColorTheme, color_themes};

use std::collections::HashSet;

fn disp_choose_files() -> Vec<String> {
    let mut chooser = fltk::dialog::NativeFileChooser::new(
        fltk::dialog::FileDialogType::BrowseMultiFile
    );

    chooser.set_title("Choose PDFs");
    chooser.set_filter("*.pdf");
    chooser.show();
    if chooser.filenames().get(0).is_none() {
        return vec![]
    }

    let filenames: Vec<String> = chooser.filenames().into_iter().map(|c| {
        c.into_os_string().into_string().unwrap()
    }).collect();

    filenames
}

fn disp_choose_output() -> Option<String> {
    let mut chooser = fltk::dialog::NativeFileChooser::new(
        fltk::dialog::FileDialogType::BrowseDir
    );
    chooser.set_title("Choose Output Directory");
    chooser.show();

    if chooser.filenames().get(0).is_none() {
        return None
    }

    let filenames: Vec<String> = chooser.filenames().into_iter().map(|c| {
        if c.is_file() {
            c.parent().unwrap().as_os_str().to_os_string().into_string().unwrap()
        } else {
            c.into_os_string().into_string().unwrap()
        }
    }).collect();
    Some(filenames.get(0).unwrap().to_owned())
}

fn px_to_mm(dpi: f64, px_size: usize) -> f64 {
    let i = 25.4;
    let one_mm = i / dpi;
    let mm_size = (px_size as f64) * one_mm;
    (mm_size * 100.0).round() / 100.0
}

fn get_image_offsets(width: usize, height: usize) -> (f64, f64) {
    let image_mm_x = px_to_mm(300.0, width);
    let image_mm_y = px_to_mm(300.0, height);
    let offset_x = (100.0 - image_mm_x) / 2.0;
    let offset_y = (150.0 - image_mm_y) / 2.0;
    (offset_x, offset_y)
}

fn save_pdf(file_names: &HashSet<String>, output_dir: &String) -> Result<(), &'static str> {
    let mut images: Vec<ImageXObject> = vec![];
    let file_amount = file_names.len();

    for name in file_names.into_iter() {
        images.extend(extract_images(name.as_str()));
    }

    let (doc, page1, layer1) = PdfDocument::new("Labels", Mm(100.0), Mm(150.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    let mut layers: Vec<PdfLayerReference> = vec![current_layer];

    for i in 0..(file_amount-1) {
        let t = format!("Page {}", i+2);
        let (page2, layer_i) = doc.add_page(Mm(100.0), Mm(150.0), t);
        let current_layer = doc.get_page(page2).get_layer(layer_i);
        layers.extend(vec![current_layer]);
    }

    for (index, i) in images.into_iter().enumerate() {
        let o = i.clone();
        let cur_layer = layers.get(index).unwrap().to_owned();
        let img = Image::from(i);

        let orig_rot = ImageRotation::default();

        let rot = if o.width.0 > o.height.0 {
            ImageRotation {
                angle_ccw_degrees: 90.0,
                rotation_center_x: Px(o.width.0 / 2),
                rotation_center_y: Px(o.height.0 / 2),
            }
        } else {
            orig_rot
        };

        let ot = ImageTransform::default();
        let (offset_x, offset_y) = get_image_offsets(o.width.0, o.height.0);
        let x = ot.translate_x.unwrap_or_default().0 + offset_x;
        let y = ot.translate_y.unwrap_or_default().0 + offset_y;

        let t = ImageTransform {
            translate_x: Some(Mm(x)),
            translate_y: Some(Mm(y)),
            rotate: Some(rot),
            scale_x: ot.scale_x,
            scale_y: ot.scale_y,
            dpi: ot.dpi
        };
        img.add_to_layer(cur_layer, t);
    }
    let local: DateTime<Local> = Local::now();
    let date = format!("{}", local.format("%Y%m%d%H%M"));
    let output_filename = format!("{}/labels-{}.pdf", output_dir, date);
    doc.save(&mut BufWriter::new(File::create(output_filename).unwrap())).unwrap();

    Ok(())
}

#[derive(Clone)]
pub enum Message {
    Labels(Vec<String>),
    OutputDir(String),
    Clear,
    Done,
}

pub fn center() -> (i32, i32) {
    (
        (app::screen_size().0 / 2.0) as i32,
        (app::screen_size().1 / 2.0) as i32,
    )
}

fn main() -> std::io::Result<()> {
    let app = app::App::default().with_scheme(app::Scheme::Gleam);
    let theme = ColorTheme::new(color_themes::BLACK_THEME);
    theme.apply();

    let (s, r) = app::channel::<Message>();
    let s1 = s.clone();
    let s2 = s.clone();
    let s3 = s.clone();

    let (screen_width, screen_height) = (400, 400);
    let (screen_x, screen_y) = (
        (app::screen_size().0 / 2.0) as i32 - (screen_width/2),
        (app::screen_size().1 / 2.0) as i32 - (screen_height/2),
    );
    let button_y = 15;
    let mut wind = Window::new(screen_x, screen_y, screen_width, screen_height, "Prepare Labels");
    let _frame = Frame::new(0, 0, 400, 300, "");
    let mut labels_but = Button::new(10, button_y, 110, 40, "Select Labels");
    let mut output_dir_but = Button::new(145, button_y, 110, 40, "Select Output");
    let mut convert_but = Button::new(280, button_y, 110, 40, "Convert!");
    let mut file_name_txt = TextDisplay::new(5, 80, 190, 315, "PDFs");
    let mut output_dir_txt = TextDisplay::new(195, 80, 200, 40, "Output Directory");
    let mut clear_but = Button::new(240, 200, 110, 80, "Clear");
    let file_name_buf = TextBuffer::default();
    let output_buf = TextBuffer::default();
    file_name_txt.set_buffer(file_name_buf);
    output_dir_txt.set_buffer(output_buf);
    wind.end();
    wind.show();

    labels_but.set_callback(move |_| {
        let s = s.clone();
        let a = disp_choose_files();
        s.send(Message::Labels(a));
    });

    output_dir_but.set_callback(move |_| {
        let a = match disp_choose_output() {
            Some(c) => c,
            None => String::new()
        };
        s1.send(Message::OutputDir(a));
    });

    convert_but.set_callback(move |_| {
        s2.send(Message::Done);
    });

    clear_but.set_callback(move |_| {
        s3.send(Message::Clear);
    });

    //let mut file_names: Vec<String> = vec![];
    let mut file_names = HashSet::new();
    let mut file_names_set = false;
    let mut output_set = false;
    let mut output_directory: String = String::new();

    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                Message::Labels(labels) => {
                    file_names.extend(labels);
                    let fns: Vec<String> = file_names.clone().into_iter().map(|s| {
                        s.split("/").last().unwrap().to_owned()
                    }).collect();
                    let text = fns.join("\n");
                    let mut buf = TextBuffer::default();
                    buf.set_text(&text);
                    file_name_txt.set_buffer(buf);

                    file_names_set = true;
                },
                Message::OutputDir(output_dir) => {
                    output_directory = output_dir.clone();
                    let mut buf = TextBuffer::default();
                    buf.set_text(&output_dir);
                    output_dir_txt.set_buffer(buf);
                    output_set = true;
                },
                Message::Clear => {
                    output_directory = String::new();
                    file_names.clear();
                    file_names_set = false;
                    output_set = false;
                    let buf = TextBuffer::default();
                    let buf2 = TextBuffer::default();
                    file_name_txt.set_buffer(buf);
                    output_dir_txt.set_buffer(buf2);
                }
                Message::Done => {
                    if !output_set || !file_names_set {
                        let mut messages: Vec<String> = vec![];

                        if !file_names_set {
                            messages.extend(vec!["PDFs".to_owned()]);
                        }

                        if !output_set {
                            messages.extend(vec!["output directory".to_owned()]);
                        }

                        let message = messages.join(" and ");

                        fltk::dialog::alert(
                            center().0 - 200,
                            center().1 - 100,
                            &format!("You need to select your {}.", message),
                        );
                    } else {
                        save_pdf(&file_names, &output_directory).unwrap();
                        fltk::dialog::message(
                            center().0 - 200,
                            center().1 - 100,
                            "Done"
                        )
                    }
                }
            }
        }
    }
    Ok(())
}