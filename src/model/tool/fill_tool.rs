use super::Tool;

pub struct FillTool {}

impl Tool for FillTool
{
    fn get_icon_name(&self) -> &'static str { "edit-select" }

    /* 
    fn add_tool_page(&self, window: &ApplicationWindow,parent: &mut gtk4::Box)
    {
        parent.append(&gtk4::Label::builder().label("FillTool").build());
    }*/
}