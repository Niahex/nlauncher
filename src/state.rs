use gtk::{gio, CustomFilter, FilterListModel, SingleSelection, prelude::*};

pub struct LauncherState {
    pub app_list_store: gio::ListStore,
    pub filtered_list_model: FilterListModel,
    pub selection_model: SingleSelection,
    pub app_filter: CustomFilter,
}

impl LauncherState {
    pub fn new(app_list_store: gio::ListStore, search_text_provider: impl Fn() -> String + 'static) -> Self {
        let app_filter = CustomFilter::new(move |obj| {
            let app_info = obj.downcast_ref::<gio::AppInfo>().unwrap();
            let search_text = search_text_provider().to_lowercase();
            app_info.name().to_lowercase().contains(&search_text)
        });

        let filtered_list_model = FilterListModel::new(Some(app_list_store.clone()), Some(app_filter.clone()));
        let selection_model = SingleSelection::new(Some(filtered_list_model.clone()));

        Self {
            app_list_store,
            filtered_list_model,
            selection_model,
            app_filter,
        }
    }
}