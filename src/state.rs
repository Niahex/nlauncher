use gtk::{gio, CustomFilter, FilterListModel, SingleSelection, prelude::*};
use std::rc::Rc;
use std::cell::RefCell;

pub struct LauncherState {
    pub app_list_store: gio::ListStore,
    pub filtered_list_model: FilterListModel,
    pub selection_model: SingleSelection,
    pub app_filter: CustomFilter,
    pub search_query: Rc<RefCell<String>>,
}

impl LauncherState {
    pub fn new(app_list_store: gio::ListStore) -> Self {
        let search_query = Rc::new(RefCell::new(String::new()));
        let query_clone = search_query.clone();

        let app_filter = CustomFilter::new(move |obj| {
            let query = query_clone.borrow();
            if query.is_empty() {
                return true;
            }
            
            let app_info = obj.downcast_ref::<gio::AppInfo>().unwrap();
            // Optimization: check if name contains query.
            // app_info.name() returns GString (not Option), which derefs to &str
            app_info.name().to_lowercase().contains(query.as_str())
        });

        let filtered_list_model = FilterListModel::new(Some(app_list_store.clone()), Some(app_filter.clone()));
        let selection_model = SingleSelection::new(Some(filtered_list_model.clone()));

        Self {
            app_list_store,
            filtered_list_model,
            selection_model,
            app_filter,
            search_query,
        }
    }
}