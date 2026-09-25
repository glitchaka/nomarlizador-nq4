pub fn rgba_icon(size: u32) -> Vec<u8> {
    let mut pixels = vec![0_u8; (size * size * 4) as usize];

    let scale = size as f32 / 256.0;
    let background = [25, 28, 34, 255];
    let accent = [40, 190, 180, 255];
    let white = [245, 248, 250, 255];

    for y in 0..size {
        for x in 0..size {
            let px = x as f32 / scale;
            let py = y as f32 / scale;

            if rounded_rect_contains(px, py, 10.0, 10.0, 246.0, 246.0, 52.0) {
                put_pixel(&mut pixels, size, x, y, background);
            }

            if point_in_polygon(
                px,
                py,
                &[
                    (50.0, 74.0),
                    (158.0, 48.0),
                    (210.0, 100.0),
                    (184.0, 186.0),
                    (76.0, 208.0),
                    (42.0, 156.0),
                ],
            ) {
                put_pixel(&mut pixels, size, x, y, accent);
            }

            if circle_contains(px, py, 163.0, 81.0, 13.0) {
                put_pixel(&mut pixels, size, x, y, background);
            }

            let left_note_stem = rounded_rect_contains(px, py, 102.0, 88.0, 118.0, 163.0, 7.0);
            let right_note_stem = rounded_rect_contains(px, py, 151.0, 81.0, 165.0, 149.0, 6.0);
            let left_note = circle_contains(px, py, 98.0, 166.0, 20.0);
            let right_note = circle_contains(px, py, 146.5, 151.0, 18.5);
            let beam = point_in_polygon(
                px,
                py,
                &[(112.0, 88.0), (161.0, 78.0), (161.0, 96.0), (112.0, 106.0)],
            );

            if left_note_stem || right_note_stem || left_note || right_note || beam {
                put_pixel(&mut pixels, size, x, y, white);
            }
        }
    }

    pixels
}

fn put_pixel(pixels: &mut [u8], size: u32, x: u32, y: u32, rgba: [u8; 4]) {
    let index = ((y * size + x) * 4) as usize;
    pixels[index..index + 4].copy_from_slice(&rgba);
}

fn circle_contains(x: f32, y: f32, cx: f32, cy: f32, radius: f32) -> bool {
    let dx = x - cx;
    let dy = y - cy;
    dx * dx + dy * dy <= radius * radius
}

fn rounded_rect_contains(
    x: f32,
    y: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    radius: f32,
) -> bool {
    if x < left || x > right || y < top || y > bottom {
        return false;
    }

    let inner_left = left + radius;
    let inner_right = right - radius;
    let inner_top = top + radius;
    let inner_bottom = bottom - radius;

    if (x >= inner_left && x <= inner_right) || (y >= inner_top && y <= inner_bottom) {
        return true;
    }

    let cx = if x < inner_left { inner_left } else { inner_right };
    let cy = if y < inner_top { inner_top } else { inner_bottom };
    circle_contains(x, y, cx, cy, radius)
}

fn point_in_polygon(x: f32, y: f32, points: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let mut j = points.len() - 1;

    for i in 0..points.len() {
        let (xi, yi) = points[i];
        let (xj, yj) = points[j];

        let crosses = ((yi > y) != (yj > y))
            && (x < (xj - xi) * (y - yi) / ((yj - yi).abs().max(f32::EPSILON)) + xi);

        if crosses {
            inside = !inside;
        }

        j = i;
    }

    inside
}
