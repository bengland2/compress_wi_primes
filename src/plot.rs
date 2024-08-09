use plotters::prelude::*;
//use plotters::coord::types::RangedCoordf32;
use std::env;
use std::string::ToString;
use partial_min_max;

pub fn vec_u32_to_f64( v_in : &Vec<u32> ) -> Vec<f64> {
    let mut v_out : Vec<f64> = vec![];
    for el in v_in {
        v_out.push(*el as f64);
    }
    v_out
}

pub fn plot_histogram_u32(filename : &str, plot_name : &str, x_label : &str, y_label : &str, hist_vec : &Vec<u32> ) -> Result<(), Box<dyn std::error::Error>> {
    let hist_as_f64 = vec_u32_to_f64(hist_vec);
    plot_histogram_f64(filename, plot_name, x_label, y_label, &hist_as_f64)
}

pub fn plot_histogram_f64( filename : &str, plot_name : &str, x_label : &str, y_label : &str, hist_vec : &[f64] ) -> Result<(), Box<dyn std::error::Error>> {
    let binding = std::path::MAIN_SEPARATOR.to_string();
    let sep = binding.as_str();
    let pathname = env::var("TMPDIR").unwrap() + sep + filename;
    println!("creating file {}", pathname);
    let root = BitMapBackend::new(pathname.as_str(), (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let root = root.margin(10, 10, 10, 10);

    // After this point, we should be able to construct a chart context
    // determine range of Y-axis

    let raw_min = hist_vec.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let raw_max = hist_vec.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let y_min = partial_min_max::max(*raw_min, 0.0);
    let y_max = raw_max.ceil();
    let mut chart = ChartBuilder::on(&root)
        // Set the caption of the chart
        .caption(plot_name, ("sans-serif", 40).into_font())
        // Set the size of the label region
        .x_label_area_size(30)
        .y_label_area_size(50)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_cartesian_2d(0f32.. hist_vec.len() as f32, y_min..y_max)?;

    // Then we can draw a mesh
    chart
        .configure_mesh()
        // We can customize the maximum number of labels allowed for each axis
        .x_labels(10)
        .y_labels(5)
        .x_label_offset(10)
        .y_label_offset(10)
        .x_desc(x_label)
        .y_desc(y_label)
        // We can also change the format of the label text
        .y_label_formatter(&|x| format!("{:.1}", x))
        .draw().unwrap();

    // And we can draw something in the drawing area
    let mut series : Vec<(f32, f64)>  = vec![];
    //for k in 0..hist_vec.len() {
    for (k, h) in hist_vec.iter().enumerate() {
        series.push((k as f32, *h));
    }
    chart.draw_series(LineSeries::new( series, &RED ))?;
/*
    // Similarly, we can draw point series
    chart.draw_series(PointSeries::of_element(series.iter(), series.len(),
        &RED,
        &|c, s, st| {
            return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
        },
    ))?;
*/
    root.present()?;
    Ok(())
}