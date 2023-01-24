use std::sync::{Arc};
use std::collections::HashMap;

use pdf::file::File;
use pdf::object::*;

use printpdf::{ImageXObject, Px};


fn get_image(o: &RcRef<XObject>) -> Option<&pdf::object::ImageXObject>{
    match **o {
        XObject::Image(ref im) => Some(im),
        _ => None
    }
}

fn get_xobj(data: Arc<[u8]>, image: &pdf::object::ImageXObject) -> printpdf::ImageXObject {
    let a = data.into_iter().as_slice();
    let width = image.width as usize;
    let height = image.height as usize;

    let color_space = match image.color_space.as_ref().unwrap() {
        pdf::object::ColorSpace::DeviceGray => printpdf::ColorSpace::Greyscale,
        pdf::object::ColorSpace::DeviceRGB => printpdf::ColorSpace::Rgb,
        _ => printpdf::ColorSpace::Greyscale,
    };

    let bits = match image.bits_per_component.unwrap() {
        1 => printpdf::ColorBits::Bit1,
        8 => printpdf::ColorBits::Bit8,
        16 => printpdf::ColorBits::Bit16,
        _ => printpdf::ColorBits::Bit8,
    };

    ImageXObject {
        width: Px(width),
        height: Px(height),
        color_space: color_space,
        bits_per_component: bits,
        interpolate: false,
        image_data: Vec::from(a),
        image_filter: None, /* does not work yet */
        clipping_bbox: None, /* doesn't work either, untested */
    }
}

pub fn extract_images(path: &str) -> Vec<printpdf::ImageXObject> {
    let file = File::<Vec<u8>>::open(&path).unwrap();
    let mut images: Vec<_> = vec![];
    let mut fonts = HashMap::new();

    for page in file.pages() {
        let page = page.unwrap();
        let resources = page.resources().unwrap();
        for (i, font) in resources.fonts.values().enumerate() {
            let name = match &font.name {
                Some(name) => name.as_str().into(),
                None => i.to_string(),
            };
            fonts.insert(name, font.clone());
        }
        images.extend(resources.xobjects.iter().map(|(_name, &r)| file.get(r).unwrap())
            .filter(|o| matches!(**o, XObject::Image(_)))
        );
    }

    let extracted_images: Vec<printpdf::ImageXObject> = images.iter().enumerate().filter_map(|(_i,o)| {
        let image = get_image(o);

        if image.is_none() {
            None
        } else {
            let img = image.unwrap();
            let (data, _filter) = img.raw_image_data(&file).unwrap();
            Some(get_xobj(data, img))
        }
    }).collect();
    extracted_images
}