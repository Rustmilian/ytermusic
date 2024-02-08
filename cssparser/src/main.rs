use cssparser::*;

fn main() {
    add_stylesheet("base.css");
    
    println!("{:?}",get_style("playbar-progress", &["progress","playing"]));
}
