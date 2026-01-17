mod feed_list_item;
mod model;
mod view;

pub mod prelude {
    pub use super::FeedList;
}

use feed_list_item::FeedListItem;
use news_flash::models::{CategoryID, PluginCapabilities, UnifiedMapping, Url};

use crate::{
    prelude::*,
    ui::{
        feeds_list::{
            model::{FeedListModelData, FeedOrCategory},
            view::FeedListViewData,
        },
        tooltip,
    },
};
use std::{sync::Arc, time::Duration};

use tokio::{sync::mpsc::UnboundedSender, time::Instant};

pub struct FeedList {
    config: Arc<Config>,
    message_sender: UnboundedSender<Message>,

    view_data: FeedListViewData,
    model_data: FeedListModelData,

    is_focused: bool,
    last_sync: Instant,
}

impl FeedList {
    pub fn new(
        config: Arc<Config>,
        news_flash_utils: Arc<NewsFlashUtils>,
        message_sender: UnboundedSender<Message>,
    ) -> Self {
        Self {
            config: config.clone(),
            message_sender,
            model_data: FeedListModelData::new(news_flash_utils.clone()),
            view_data: FeedListViewData::new(&config),
            is_focused: false,
            last_sync: Instant::now(),
        }
    }
    pub(super) fn update_tooltip(
        &self,
        now_selected: Option<&FeedListItem>,
    ) -> color_eyre::Result<()> {
        if let Some(item) = now_selected {
            tooltip(
                &self.message_sender,
                item.to_tooltip(&self.config).as_str(),
                TooltipFlavor::Info,
            )?;
        }

        Ok(())
    }

    pub(super) fn selected(&self) -> Option<FeedListItem> {
        self.view_data.tree_state().selected().last().cloned()
    }

    fn generate_articles_selected_command(&self) -> color_eyre::Result<()> {
        if let Some(selected) = self.selected() {
            match selected.try_into() {
                Ok(article_filter) => {
                    self.message_sender
                        .send(Message::Event(Event::ArticlesSelected(article_filter)))?;
                }
                Err(err) => {
                    tooltip(
                        &self.message_sender,
                        err.to_string().as_str(),
                        TooltipFlavor::Warning,
                    )?;
                }
            }
        };

        Ok(())
    }

    fn set_current_read(&mut self) -> color_eyre::Result<()> {
        use FeedListItem::*;
        if let Some(selected) = self.selected().as_ref() {
            match selected {
                All => self.model_data.set_all_read()?,
                Feed(feed) => self.model_data.set_feed_read(vec![feed.feed_id.clone()])?,
                Category(category) => self
                    .model_data
                    .set_category_read(vec![category.category_id.clone()])?,
                Tag(tag) => self.model_data.set_tag_read(vec![tag.tag_id.clone()])?,
                Tags => {}
                Categories | Query(_) => {
                    // reroute to article list
                    self.message_sender
                        .send(Message::Command(Command::ActionSetRead(
                            ActionTarget::ArticleList,
                            ActionScope::All,
                        )))?;
                }
            }

            tooltip(
                &self.message_sender,
                format!("set articles in {} to read", selected).as_str(),
                TooltipFlavor::Info,
            )?;
        }

        Ok(())
    }

    async fn rename_current(&mut self, name: String) -> color_eyre::Result<()> {
        use FeedListItem::*;
        if let Some(selected) = self.selected().as_ref() {
            match selected {
                not_supported @ (All | Tags | Query(_) | Categories) => {
                    return tooltip(
                        &self.message_sender,
                        format!("renaming not supported for {not_supported}").as_str(),
                        TooltipFlavor::Warning,
                    );
                }
                Feed(feed) => self
                    .model_data
                    .rename_feed(feed.feed_id.clone(), name.clone())?,
                Category(category) => {
                    if !self
                        .model_data
                        .features()
                        .await?
                        .contains(PluginCapabilities::MODIFY_CATEGORIES)
                    {
                        return tooltip(
                            &self.message_sender,
                            "provider does not support modifying categories",
                            TooltipFlavor::Error,
                        );
                    } else {
                        self.model_data
                            .rename_category(category.category_id.clone(), name.clone())?;
                    }
                }
                Tag(tag) => match self.model_data.get_tag_by_label(&name) {
                    Some(_) => {
                        return tooltip(
                            &self.message_sender,
                            format!("tag with name #{name} already exists").as_str(),
                            TooltipFlavor::Error,
                        );
                    }
                    None => self
                        .model_data
                        .edit_tag(tag.tag_id.clone(), name.clone(), None)?,
                },
            }

            tooltip(
                &self.message_sender,
                format!("renaming {} to {}", selected, name).as_str(),
                TooltipFlavor::Info,
            )?;
        }

        Ok(())
    }

    async fn remove_current(&mut self, remove_children: bool) -> color_eyre::Result<()> {
        use FeedListItem::*;
        if let Some(selected) = self.selected().as_ref() {
            match selected {
                not_supported @ (All | Tags | Query(_) | Categories) => {
                    tooltip(
                        &self.message_sender,
                        format!("removing not supported for {not_supported}").as_str(),
                        TooltipFlavor::Warning,
                    )?;
                    return Ok(());
                }
                Feed(feed) => {
                    if !self
                        .model_data
                        .features()
                        .await?
                        .contains(PluginCapabilities::ADD_REMOVE_FEEDS)
                    {
                        tooltip(
                            &self.message_sender,
                            "provider does not support modifying feeds",
                            TooltipFlavor::Error,
                        )?;
                    } else {
                        self.model_data.remove_feed(feed.feed_id.clone())?;
                    }
                }
                Category(category) => {
                    if !self
                        .model_data
                        .features()
                        .await?
                        .contains(PluginCapabilities::MODIFY_CATEGORIES)
                    {
                        tooltip(
                            &self.message_sender,
                            "provider does not support modifying categories",
                            TooltipFlavor::Error,
                        )?;
                    } else {
                        self.model_data
                            .remove_category(category.category_id.to_owned(), remove_children)?;
                    }
                }
                Tag(tag) => self.model_data.remove_tag(tag.tag_id.to_owned())?,
            }

            tooltip(
                &self.message_sender,
                format!("removing {}", selected).as_str(),
                TooltipFlavor::Info,
            )?;
        }

        Ok(())
    }

    fn target_matches(&self, target: &ActionTarget) -> bool {
        match target {
            ActionTarget::Current if self.is_focused => true,
            ActionTarget::FeedList => true,
            _ => false,
        }
    }

    fn maybe_selected_category(&self) -> Option<CategoryID> {
        let selected = self.view_data.tree_state().selected();

        self.selected()
            .and_then(|feed_list_item| match feed_list_item {
                FeedListItem::Category(category) => Some(category.category_id.to_owned()),
                _ => None,
            })
            .or_else(|| {
                selected
                    .get(selected.len().saturating_sub(2))
                    .map(|second_to_last| match second_to_last {
                        FeedListItem::Category(category) => Some(category.category_id.to_owned()),
                        _ => None,
                    })?
            })
    }

    async fn add_category(&self, name: &String) -> color_eyre::Result<()> {
        let features = self.model_data.features().await?;
        if !features.contains(
            PluginCapabilities::SUPPORT_CATEGORIES | PluginCapabilities::MODIFY_CATEGORIES,
        ) {
            tooltip(
                &self.message_sender,
                "cannot add category: provider does not support modification of categories",
                TooltipFlavor::Error,
            )?;
        }

        let mut category_id = self.maybe_selected_category();
        if category_id.is_some() && !features.intersects(PluginCapabilities::SUPPORT_SUBCATEGORIES)
        {
            category_id = None;
            tooltip(
                &self.message_sender,
                "provider does not support subcategories: creating at root level",
                TooltipFlavor::Warning,
            )?;
        } else {
            tooltip(
                &self.message_sender,
                format!("adding category {}...", name).as_str(),
                TooltipFlavor::Info,
            )?;
        }

        self.model_data
            .add_category(name.clone(), category_id)
            .await
    }

    async fn check_tag_capability(&self) -> color_eyre::Result<bool> {
        if !self
            .model_data
            .features()
            .await?
            .contains(PluginCapabilities::SUPPORT_TAGS)
        {
            tooltip(
                &self.message_sender,
                "this provider does not support tags",
                TooltipFlavor::Error,
            )?;
            return Ok(false);
        }
        Ok(true)
    }

    async fn change_feed_url(&self, url: &Url) -> color_eyre::Result<()> {
        if !self
            .model_data
            .features()
            .await?
            .contains(PluginCapabilities::EDIT_FEED_URLS)
        {
            tooltip(
                &self.message_sender,
                "provider does not support changeing the URL of a feed",
                TooltipFlavor::Error,
            )?;
            return Ok(());
        }

        let Some(FeedListItem::Feed(feed)) = self.selected() else {
            tooltip(
                &self.message_sender,
                "no feed selected",
                TooltipFlavor::Warning,
            )?;
            return Ok(());
        };

        self.model_data
            .change_feed_url(feed.feed_id.to_owned(), url.to_string())?;

        Ok(())
    }

    fn yank_feed_or_category(&mut self) -> color_eyre::Result<()> {
        match self.selected() {
            Some(FeedListItem::Feed(feed)) => self.view_data.yank_feed_or_category(
                FeedOrCategory::Feed(feed.feed_id.to_owned()),
                &self.model_data,
            ),
            Some(FeedListItem::Category(category)) => self.view_data.yank_feed_or_category(
                FeedOrCategory::Category(category.category_id.to_owned()),
                &self.model_data,
            ),
            _ => {
                tooltip(
                    &self.message_sender,
                    "can only yank feed or category",
                    TooltipFlavor::Warning,
                )?;
            }
        }

        Ok(())
    }

    fn paste_feed_or_category(&mut self, position: PastePosition) -> color_eyre::Result<()> {
        use FeedListItem::*;

        let mut yanked_unified_mapping = self.view_data.take_yanked_unified_mapping();

        let Some(to_unified_mapping) = yanked_unified_mapping.as_mut() else {
            return tooltip(
                &self.message_sender,
                "cannot paste as there is no item yanked",
                TooltipFlavor::Warning,
            );
        };

        // manual clone
        let from_unified_mapping = match to_unified_mapping {
            UnifiedMapping::Category(category_mapping) => {
                UnifiedMapping::Category(category_mapping.clone())
            }
            UnifiedMapping::Feed(feed_mapping) => UnifiedMapping::Feed(feed_mapping.clone()),
        };

        let Some((mut new_parent_category_id, dest_unified_mapping)) = (match self.selected() {
            Some(Category(category)) => self
                .model_data
                .category_mapping_for_category()
                .get(&category.category_id)
                .cloned()
                .map(|mapping| {
                    (
                        mapping.parent_id.to_owned(),
                        UnifiedMapping::Category(mapping),
                    )
                }),

            Some(Feed(feed)) => self
                .model_data
                .feed_mapping_for_feed()
                .get(&feed.feed_id)
                .cloned()
                .map(|mapping| {
                    (
                        mapping.category_id.to_owned(),
                        UnifiedMapping::Feed(mapping),
                    )
                }),
            _ => {
                self.view_data
                    .set_yanked_unified_mapping(yanked_unified_mapping);
                return tooltip(
                    &self.message_sender,
                    "pasting does not work here",
                    TooltipFlavor::Warning,
                );
            }
        }) else {
            self.view_data
                .set_yanked_unified_mapping(yanked_unified_mapping);
            return tooltip(
                &self.message_sender,
                "selected item has no category mapping",
                TooltipFlavor::Error,
            );
        };

        to_unified_mapping.set_sort_index(
            dest_unified_mapping
                .sort_index()
                .unwrap_or(i32::MAX)
                .saturating_add(match position {
                    PastePosition::Before => -1,
                    PastePosition::After => 1,
                }),
        );

        if let UnifiedMapping::Category(dest_category_mapping) = dest_unified_mapping
            && matches!(position, PastePosition::After)
        {
            new_parent_category_id = dest_category_mapping.category_id;
            to_unified_mapping.set_sort_index(0);
        }

        match (from_unified_mapping, to_unified_mapping) {
            (UnifiedMapping::Feed(from_feed_mapping), UnifiedMapping::Feed(to_feed_mapping)) => {
                to_feed_mapping.category_id = new_parent_category_id;
                self.model_data
                    .move_feed(from_feed_mapping, to_feed_mapping.to_owned())?;
            }

            (UnifiedMapping::Category(_), UnifiedMapping::Category(to_category_mapping)) => {
                to_category_mapping.parent_id = new_parent_category_id;
                self.model_data
                    .move_category(to_category_mapping.to_owned())?;
            }

            _ => unreachable!(),
        }

        Ok(())
    }
}

impl MessageReceiver for FeedList {
    async fn process_command(&mut self, message: &Message) -> color_eyre::Result<()> {
        if matches!(message, Message::Event(Event::Tick)) {
            if let Some(sync_interval) = self.config.sync_every_minutes
                && self
                    .last_sync
                    .checked_add(Duration::from_mins(sync_interval))
                    .map(|time_to_sync| Instant::now() > time_to_sync)
                    .unwrap_or(false)
            {
                self.message_sender
                    .send(Message::Command(Command::FeedListSync))?;
                self.last_sync = Instant::now();
            }

            return Ok(());
        }

        // get selection before
        let selected_before_item = self.selected().clone();
        let selected_before = self.view_data.tree_state().selected().to_vec();
        let mut model_needs_update = false;
        let mut view_needs_update = false;

        // commands
        if let Message::Command(command) = message {
            use Command as C;
            let mut handle_command = false;

            let Some(command) = (match command {
                C::In(Panel::FeedList, command) => {
                    handle_command = true;
                    Some(*command.to_owned())
                }
                C::In(..) => None,
                command => {
                    handle_command = self.is_focused;
                    Some(command.to_owned())
                }
            }) else {
                return Ok(());
            };

            match command {
                C::NavigateUp if handle_command => {
                    self.view_data.tree_state_mut().key_up();
                    view_needs_update = true;
                }
                C::NavigateDown if handle_command => {
                    self.view_data.tree_state_mut().key_down();
                    view_needs_update = true;
                }
                C::NavigateRight if handle_command => {
                    self.view_data.tree_state_mut().key_right();
                    view_needs_update = true;
                }
                C::NavigateLeft if handle_command => {
                    self.view_data.tree_state_mut().key_left();
                    view_needs_update = true;
                }
                C::NavigateFirst if handle_command => {
                    self.view_data.tree_state_mut().select_first();
                    view_needs_update = true;
                }
                C::NavigateLast if handle_command => {
                    self.view_data.tree_state_mut().select_last();
                    view_needs_update = true;
                }
                C::FeedListToggleExpand if handle_command => {
                    self.view_data.tree_state_mut().toggle_selected();
                    view_needs_update = true;
                }
                C::NavigatePageDown if handle_command => {
                    self.view_data
                        .tree_state_mut()
                        .scroll_down(self.config.input_config.scroll_amount);
                    view_needs_update = true;
                }
                C::NavigatePageUp if handle_command => {
                    self.view_data
                        .tree_state_mut()
                        .scroll_up(self.config.input_config.scroll_amount);
                    view_needs_update = true;
                }
                show_command @ (C::Show(ActionTarget::FeedList, scope)
                | C::Show(ActionTarget::Current, scope))
                    if matches!(show_command, C::Show(ActionTarget::FeedList, _))
                        || handle_command =>
                {
                    *self.view_data.scope_mut() = scope;
                    view_needs_update = true;
                }

                C::ActionSetRead(target, action_scope) if self.target_matches(&target) => {
                    match action_scope {
                        ActionScope::All => self.model_data.set_all_read()?,
                        ActionScope::Current => self.set_current_read()?,
                        query_scope @ ActionScope::Query(_) => {
                            // don't know how to handle article query => "rerouting" to article
                            // list
                            self.message_sender
                                .send(Message::Command(Command::ActionSetRead(
                                    ActionTarget::ArticleList,
                                    query_scope.to_owned(),
                                )))?;
                        }
                    }
                }

                C::FeedListFeedAdd(url, name) => {
                    let features = self.model_data.features().await?;
                    if !features.contains(PluginCapabilities::ADD_REMOVE_FEEDS) {
                        tooltip(
                            &self.message_sender,
                            "provider does not support adding feeds",
                            TooltipFlavor::Error,
                        )?;
                    } else {
                        self.model_data.add_feed(
                            url.as_ref()
                                .ok_or(color_eyre::eyre::eyre!("no url defined"))?
                                .to_owned(),
                            name.clone(),
                            self.maybe_selected_category(),
                        )?;
                        tooltip(&self.message_sender, "adding feed...", TooltipFlavor::Info)?;
                    }
                }

                C::FeedListCategoryAdd(name) => {
                    self.add_category(&name).await?;
                }

                C::FeedListFeedChangeUrl(url) => {
                    self.change_feed_url(
                        url.as_ref()
                            .ok_or(color_eyre::eyre::eyre!("no url defined"))?,
                    )
                    .await?
                }

                C::TagAdd(name, color) if self.check_tag_capability().await? => {
                    if self.model_data.tags().iter().any(|tag| *tag.label == *name) {
                        tooltip(
                            &self.message_sender,
                            format!("tag with name {} already exists", name).as_str(),
                            TooltipFlavor::Error,
                        )?;
                    } else {
                        self.model_data.add_tag(&name, &color).await?;
                    }
                }

                C::FeedListTagChangeColor(color) => match self.selected() {
                    Some(FeedListItem::Tag(tag)) => {
                        self.model_data.edit_tag(
                            tag.tag_id.to_owned(),
                            tag.label.to_owned(),
                            Some(color.to_owned()),
                        )?;
                    }
                    _ => tooltip(
                        &self.message_sender,
                        "select a tag to change its color",
                        TooltipFlavor::Error,
                    )?,
                },
                C::FeedListSync => {
                    tooltip(&self.message_sender, "syncing all", TooltipFlavor::Info)?;
                    self.model_data.sync()?;
                }

                C::FeedListRenameEntity(name) => {
                    self.rename_current(name.to_owned()).await?;
                }

                C::FeedListRemoveEntity => {
                    self.remove_current(false).await?;
                }

                C::FeedListRemoveEntityWithChildren => {
                    self.remove_current(true).await?;
                }

                C::FeedListYankFeedOrCategory => {
                    self.yank_feed_or_category()?;
                    view_needs_update = true;
                }

                C::FeedListPasteFeedOrCategory(position) => {
                    self.paste_feed_or_category(position)?;
                }

                _ => {}
            }
        };

        // messages
        if let Message::Event(event) = message {
            use Event as E;
            match event {
                E::ApplicationStarted => {
                    model_needs_update = true;
                }

                E::ApplicationStateChanged(state) => {
                    self.is_focused = *state == AppState::FeedSelection;
                }

                E::AsyncFeedAddFinished(feed) => {
                    tooltip(
                        &self.message_sender,
                        format!("successfully added feed {}, fetching feed now", feed.label)
                            .as_str(),
                        TooltipFlavor::Info,
                    )?;
                    self.model_data.fetch_feed(feed.feed_id.to_owned())?;
                }

                E::AsyncFeedFetchFinished(..) => {
                    // sync to get new articles
                    self.message_sender
                        .send(Message::Command(Command::FeedListSync))?;
                }

                E::AsyncSyncFinished(_) => {
                    tooltip(
                        &self.message_sender,
                        "finished syncing",
                        TooltipFlavor::Info,
                    )?;
                    model_needs_update = true;
                }

                E::AsyncCategoryAddFinished(category) => {
                    tooltip(
                        &self.message_sender,
                        format!("successfully added category {}", category.label).as_str(),
                        TooltipFlavor::Info,
                    )?;
                    model_needs_update = true;
                    // self.model_data.sync()?;
                }

                E::AsyncFeedMoveFinished | E::AsyncCategoryMoveFinished => {
                    tooltip(&self.message_sender, "move successful", TooltipFlavor::Info)?;
                    // self.model_data.sync()?;
                    model_needs_update = true;
                }

                E::AsyncRenameFeedFinished(_)
                | E::AsyncCategoryRenameFinished(_)
                | E::AsyncTagEditFinished(_) => {
                    tooltip(
                        &self.message_sender,
                        "successfully changed",
                        TooltipFlavor::Info,
                    )?;
                    model_needs_update = true;
                    // self.model_data.sync()?;
                }

                E::AsyncFeedRemoveFinished
                | E::AsyncCategoryRemoveFinished
                | E::AsyncTagRemoveFinished => {
                    tooltip(
                        &self.message_sender,
                        "removal successful",
                        TooltipFlavor::Info,
                    )?;
                    model_needs_update = true;
                    // self.model_data.sync()?;
                }

                event if event.caused_model_update() => model_needs_update = true,
                _ => {}
            }
        }

        let selected_after_item = self.selected();

        if model_needs_update {
            self.model_data.update().await?;
            self.view_data
                .update(&self.config, &self.model_data)
                .await?;
            self.view_data.ensure_sensible_selection(&selected_before);
            self.message_sender
                .send(Message::Command(Command::Redraw))?;
        } else if view_needs_update {
            self.view_data
                .update(&self.config, &self.model_data)
                .await?;
            self.view_data.ensure_sensible_selection(&selected_before);
            self.message_sender
                .send(Message::Command(Command::Redraw))?;
        }

        if selected_before_item.as_ref() != selected_after_item.as_ref() {
            // if self.view_data.yanked_unified_mapping().is_some() {
            //     view_needs_update = true;
            // }
            self.update_tooltip(selected_after_item.as_ref())?;
            self.generate_articles_selected_command()?;
        }

        Ok(())
    }
}
