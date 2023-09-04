use std::{
    cmp::{max, min},
    collections::HashMap,
    f64::consts,
};

use icy_engine::Rectangle;

use super::Position;

// Code from Pablodraw - I liked the approach and I needed a better ellipse drawing algorithm.
// translated from https://github.com/cwensley/pablodraw/blob/main/Source/Pablo/Drawing/ScanLines.cs

pub struct ScanLines {
    line_thickness: i32,
    rows: HashMap<i32, ScanRow>,
}

impl ScanLines {
    const RAD2DEG: f64 = 180.0 / consts::PI;
    // const deg2rad: f64 = consts::PI / 180.0;

    pub fn add(&mut self, x: i32, y: i32) {
        if let Some(row) = self.rows.get_mut(&y) {
            row.add(x);
        } else {
            let mut row = ScanRow::new();
            row.add(x);
            self.rows.insert(y, row);
        }
    }

    pub fn add_ellipse(&mut self, rectangle: Rectangle) {
        let mut rw = rectangle.size.width;
        let mut rh = rectangle.size.height;

        if rw < 2 {
            rw = 2;
        }
        if rh < 2 {
            rh = 2;
        }

        if (rw % 2) == 0 {
            rw += 1;
        }
        if (rh % 2) == 0 {
            rh += 1;
        }

        let mx = rectangle.start.x + rw / 2;
        let my = rectangle.start.y + rh / 2;

        self.add_ellipse_internal(mx, my, rw / 2, rh / 2 /*, 0, 360*/);
    }

    fn add_ellipse_internal(
        &mut self,
        x: i32,
        y: i32,
        radius_x: i32,
        radius_y: i32, /*, mut start_angle: i32, mut end_angle: i32*/
    ) {
        // check if valid angles
        //if start_angle > end_angle {
        //    std::mem::swap(&mut start_angle, &mut end_angle);
        //}

        let radius_x = max(1, radius_x);
        let radius_y = max(1, radius_y);

        let dx = radius_x * 2;
        let dy = radius_y * 2;
        let b1 = dy & 1;
        let mut stop_x = 4 * (1 - dx) * dy * dy;
        let mut stop_y = 4 * (b1 + 1) * dx * dx; // error increment
        let mut err = stop_x + stop_y + b1 * dx * dx; // error of 1 step

        let mut xoffset = radius_x;
        let mut yoffset = 0;
        let inc_x = 8 * dx * dx;
        let inc_y = 8 * dy * dy;

        let aspect = (radius_x as f64) / (radius_y as f64);

        // calculate horizontal fill angle
        let horizontal_angle = if radius_x < radius_y {
            90.0 - (45.0 * aspect)
        } else {
            45.0 / aspect
        };
        let horizontal_angle = horizontal_angle.round() as i32;

        loop {
            let e2 = 2 * err;
            let angle = ((yoffset as f64 * aspect / (xoffset as f64)).atan() * ScanLines::RAD2DEG)
                .round() as i32;
            self.symmetry_scan(
                x, y, /*start_angle, end_angle, */ xoffset,
                yoffset, /*, angle, angle <= horizontal_angle*/
            );
            if (angle - horizontal_angle).abs() < 1 {
                self.symmetry_scan(
                    x, y, /*start_angle, end_angle,*/ xoffset,
                    yoffset, /*, angle, angle > horizontal_angle*/
                );
            }

            if e2 <= stop_y {
                yoffset += 1;
                stop_y += inc_x;
                err += stop_y;
            }
            if e2 >= stop_x {
                xoffset -= 1;
                stop_x += inc_y;
                err += stop_x;
            }
            if xoffset < 0 {
                break;
            }
        }

        xoffset += 1;
        while yoffset < radius_y {
            let angle = ((yoffset as f64 * aspect / (xoffset as f64)).atan() * ScanLines::RAD2DEG)
                .round() as i32;
            self.symmetry_scan(
                x, y, /*start_angle, end_angle, */ xoffset,
                yoffset, /*, angle as i32, angle <= horizontal_angle*/
            );
            if angle == horizontal_angle {
                self.symmetry_scan(
                    x, y, /*start_angle, end_angle,*/ xoffset,
                    yoffset, /*, angle as i32, angle > horizontal_angle*/
                );
            }
            yoffset += 1;
        }
    }

    fn add_horizontal(&mut self, x: i32, y: i32, count: i32) {
        if count > 0 {
            for i in 0..count {
                self.add(x + i, y);
            }
        } else {
            for i in (0..count).rev() {
                self.add(x + i, y);
            }
        }
    }

    /*fn add_vertical(&mut self, x: i32, y: i32, count: i32) {
        if count > 0 {
            for i in 0..count {
                self.add(x, y + i);
            }
        } else {
            for i in (0..count).rev() {
                self.add(x, y + i);
            }
        }
    }*/

    pub fn fill<T>(&self, mut draw: T)
    where
        T: FnMut(Rectangle),
    {
        for row in &self.rows {
            let (min, max) = row.1.get_min_max();
            draw(Rectangle::from_coords(min, *row.0, max, *row.0));
        }
    }

    /* fn in_angle(angle: i32, start_angle: i32, end_angle: i32) -> bool {
        angle >= start_angle && angle <= end_angle
    }*/

    /*pub fn is_drawn(&self, point: Position) -> bool {
        if let Some(row) = self.rows.get(&point.y) {
            row.points.contains(&point.x)
        } else {
            false
        }
    }*/

    // simple https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm
    // maybe worth to explore https://en.wikipedia.org/wiki/Xiaolin_Wu%27s_line_algorithm
    pub fn add_line(&mut self, mut pos0: Position, pos1: Position) {
        let dx = (pos1.x - pos0.x).abs();
        let sx = if pos0.x < pos1.x { 1 } else { -1 };
        let dy = -(pos1.y - pos0.y).abs();
        let sy = if pos0.y < pos1.y { 1 } else { -1 };
        let mut error = dx + dy;

        loop {
            self.add(pos0.x, pos0.y);

            if pos0.x == pos1.x && pos0.y == pos1.y {
                break;
            }
            let e2 = 2 * error;
            if e2 >= dy {
                if pos0.x == pos1.x {
                    break;
                }
                error += dy;
                pos0.x += sx;
            }
            if e2 <= dx {
                if pos0.y == pos1.y {
                    break;
                }
                error += dx;
                pos0.y += sy;
            }
        }
    }

    pub fn add_rectangle(&mut self, rectangle: Rectangle) {
        for i in 0..rectangle.size.height {
            self.add_horizontal(
                rectangle.start.x,
                rectangle.start.y + i,
                rectangle.size.width,
            );
        }
    }

    /*  pub fn is_inside(&self, point: Position) -> bool {
        if let Some(row) = self.rows.get(&point.y) {
            let (min, max) = row.get_min_max();
            min <= point.x && point.x <= max
        } else {
            false
        }
    }*/

    pub fn new(line_thickness: i32) -> Self {
        ScanLines {
            line_thickness,
            rows: HashMap::new(),
        }
    }

    pub fn outline(&self) -> Vec<Rectangle> {
        let mut result = Vec::new();
        let mut lastx = 0;

        let mut rows = Vec::new();

        for i in &self.rows {
            let (min, max) = i.1.get_min_max();
            rows.push((*i.0, min, max));
        }
        rows.sort_by(|x, y| x.0.cmp(&y.0));
        // trace min edge
        for i in 0..rows.len() {
            let row = rows[i];

            let cur_y = row.0;
            let cur_x = row.1;
            let last = i == rows.len() - 1;
            let first = i == 0;

            if !first && !last {
                let nextx = rows[i + 1].1;
                if cur_x < lastx {
                    result.push(Rectangle::from_coords(cur_x, cur_y, lastx - 1, cur_y));
                } else if cur_x > lastx + 1 {
                    result.push(Rectangle::from_coords(
                        lastx + 1,
                        cur_y - 1,
                        cur_x - 1,
                        cur_y - 1,
                    ));
                    result.push(Rectangle::from_coords(cur_x, cur_y, cur_x, cur_y));
                } else {
                    result.push(Rectangle::from_coords(cur_x, cur_y, cur_x, cur_y));
                }
                if nextx > cur_x {
                    result.push(Rectangle::from_coords(cur_x, cur_y, nextx - 1, cur_y));
                }
            }
            lastx = cur_x;
        }

        // trace max edge
        for i in 0..rows.len() {
            let row = rows[i];
            let cur_y = row.0;
            let cur_x = row.2;
            let last = i == rows.len() - 1;
            let first = i == 0;

            if !first && !last {
                let nextx = rows[i + 1].2;
                if cur_x > lastx {
                    result.push(Rectangle::from_coords(lastx + 1, cur_y, cur_x, cur_y));
                } else if cur_x < lastx - 1 {
                    result.push(Rectangle::from_coords(
                        cur_x + 1,
                        cur_y - 1,
                        lastx - 1,
                        cur_y - 1,
                    ));
                    result.push(Rectangle::from_coords(cur_x, cur_y, cur_x, cur_y));
                } else {
                    result.push(Rectangle::from_coords(cur_x, cur_y, cur_x, cur_y));
                }
                if nextx < cur_x {
                    result.push(Rectangle::from_coords(nextx + 1, cur_y, cur_x, cur_y));
                }
            }
            lastx = cur_x;
        }

        // fill top/bottom
        if rows.is_empty() {
            return result;
        }
        let row = rows[0];
        result.push(Rectangle::from_coords(row.1, row.0, row.2, row.0));
        let row = rows[rows.len() - 1];
        result.push(Rectangle::from_coords(row.1, row.0, row.2, row.0));

        result
    }

    fn symmetry_scan(
        &mut self,
        x: i32,
        y: i32,
        /* start_angle: i32, end_angle: i32, */ xoffset: i32,
        yoffset: i32, /*, angle: i32, horizontal: bool*/
    ) {
        if self.line_thickness == 1 {
            //if ScanLines::in_angle(angle, start_angle, end_angle) {
            self.add(x + xoffset, y - yoffset);
            //}
            //if ScanLines::in_angle(180 - angle, start_angle, end_angle) {
            self.add(x - xoffset, y - yoffset);
            //}
            //if ScanLines::in_angle(180 + angle, start_angle, end_angle) {
            self.add(x - xoffset, y + yoffset);
            //}
            //if ScanLines::in_angle(360 - angle, start_angle, end_angle) {
            self.add(x + xoffset, y + yoffset);
            //}
        } /*else {
              let offset = self.line_thickness / 2;
              if horizontal {
                  if ScanLines::in_angle(angle, start_angle, end_angle) {
                      self.add_horizontal(x + xoffset - offset, y - yoffset, self.line_thickness);
                  }
                  if ScanLines::in_angle(180 - angle, start_angle, end_angle) {
                      self.add_horizontal(x - xoffset - offset, y - yoffset, self.line_thickness);
                  }
                  if ScanLines::in_angle(180 + angle, start_angle, end_angle) {
                      self.add_horizontal(x - xoffset - offset, y + yoffset, self.line_thickness);
                  }
                  if ScanLines::in_angle(360 - angle, start_angle, end_angle) {
                      self.add_horizontal(x + xoffset - offset, y + yoffset, self.line_thickness);
                  }
              } else {
                  if ScanLines::in_angle(angle, start_angle, end_angle) {
                      self.add_vertical(x + xoffset, y - yoffset - offset, self.line_thickness);
                  }
                  if ScanLines::in_angle(180 - angle, start_angle, end_angle) {
                      self.add_vertical(x - xoffset, y - yoffset - offset, self.line_thickness);
                  }
                  if ScanLines::in_angle(180 + angle, start_angle, end_angle) {
                      self.add_vertical(x - xoffset, y + yoffset - offset, self.line_thickness);
                  }
                  if ScanLines::in_angle(360 - angle, start_angle, end_angle) {
                      self.add_vertical(x + xoffset, y + yoffset - offset, self.line_thickness);
                  }
              }
          }*/
    }
}

struct ScanRow {
    pub points: Vec<i32>,
}

impl ScanRow {
    pub fn new() -> Self {
        ScanRow { points: Vec::new() }
    }
    pub fn get_min_max(&self) -> (i32, i32) {
        let mut min_point = i32::MAX;
        let mut max_point = i32::MIN;

        for i in &self.points {
            min_point = min(min_point, *i);
            max_point = max(max_point, *i);
        }
        (min_point, max_point)
    }
    pub fn add(&mut self, x: i32) {
        self.points.push(x);
    }
}
