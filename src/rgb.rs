/*
 * Copyright 2020 William Swartzendruber
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a
 * copy of the MPL was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct YcbcrGammaPixel {
    y: u8,
    cb: u8,
    cr: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgbLinearPixel {
    red: f64,
    green: f64,
    blue: f64,
}

pub fn rgb_linear_pixel(input: YcbcrGammaPixel) -> RgbLinearPixel {

    let y = bt1886_eotf(expand(input.y as f64 / 255.0));
    let cb = (input.cb as f64 - 128.0) / 128.0;
    let cr = (input.cr as f64 - 128.0) / 128.0;

    RgbLinearPixel {
        red:   y + 1.28033 * cr,
        green: y - 0.21482 * cb - 0.38059 * cr,
        blue:  y + 2.12798 * cb,
    }
}

fn ycbcr_gamma_pixel(rgb: RgbLinearPixel) -> YcbcrGammaPixel {
    YcbcrGammaPixel {
        y:
            (compress(bt1886_oetf(
                0.2126 * rgb.red
                + 0.7152 * rgb.green
                + 0.0722 * rgb.blue
            )) * 255.0).max(0.0).min(255.0).round() as u8,
        cb:
            ((
                -0.09991 * rgb.red
                - 0.33609 * rgb.green
                + 0.436 * rgb.blue
                + 1.0
            ) * 128.0).max(0.0).min(255.0).round() as u8,
        cr:
            ((
                0.615 * rgb.red
                - 0.55861 * rgb.green
                - 0.05639 * rgb.blue
                + 1.0
            ) * 128.0).max(0.0).min(255.0).round() as u8,
    }
}

fn bt1886_eotf(v: f64) -> f64 {
    v.powf(2.4).max(0.0).min(1.0)
}

fn bt1886_oetf(l: f64) -> f64 {
    l.powf(0.4166666666666667).max(0.0).min(1.0)
}

fn compress(value: f64) -> f64 {
    (value * 0.859375) + 0.06274509803
}

fn expand(value: f64) -> f64 {
    match value {
        v if v < 0.06274509803 => 0.0,
        v if v > 0.92156862745 => 1.0,
        _ => (value - 0.06274509803) / 0.859375,
    }
}
