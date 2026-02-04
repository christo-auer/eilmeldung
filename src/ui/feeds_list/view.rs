use super::model::FeedOrCategory;
use crate::prelude::*;
use crate::ui::feeds_list::model::FeedListModelData;

use getset::{Getters, MutGetters};
use log::info;
use news_flash::models::PluginCapabilities;
use news_flash::models::{Category, Feed, FeedMapping, NEWSFLASH_TOPLEVEL, UnifiedMapping};
use ratatui::text::{Line, Span};
use ratatui::widgets::Scrollbar;
use ratatui::widgets::{Block, Borders};
use ratatui::widgets::{StatefulWidget, Widget};
use strum::IntoEnumIterator;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use super::feed_list_item::FeedListItem;

#[derive(Getters, MutGetters)]
pub struct FeedListViewData {
    #[getset(get = "pub", get_mut = "pub")]
    scope: ArticleScope,

    #[getset(get = "pub", get_mut = "pub")]
    tree_state: TreeState<FeedListItem>,

    #[getset(get = "pub")]
    tree_items: Vec<TreeItem<'static, FeedListItem>>,

    #[getset(get = "pub")]
    yanked_unified_mapping: Option<UnifiedMapping>,

    #[get(get = "pub")]
    found_items: Vec<FeedListItem>,
}

impl FeedListViewData {
    pub fn new(config: &Config) -> Self {
        Self {
            scope: config.feed_list_scope,
            tree_state: Default::default(),
            tree_items: Default::default(),
            yanked_unified_mapping: Default::default(),
            found_items: Default::default(),
        }
    }
}

impl Widget for &mut FeedList {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let highlight_style = self.config.theme.selected();

        let tree_items = self.view_data.tree_items().clone();
        // let scroll_thumb_icon = self.config.scroll_thumb_icon.to_string();
        let tree = Tree::new(&tree_items)
            .unwrap() // TODO error handling
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(self.config.theme.eff_border(self.is_focused))
                    .title_top(self.view_data.build_title(&self.config)),
            )
            .experimental_scrollbar(Some(
                Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalLeft)
                    .symbols(self.config.scrollbar_set())
                    .style(self.config.theme.eff_border(self.is_focused)),
            ))
            .highlight_style(highlight_style);

        StatefulWidget::render(tree, area, buf, self.view_data.tree_state_mut());
    }
}

impl FeedListViewData {
    pub async fn update(
        &mut self,
        config: &Config,
        model_data: &FeedListModelData,
        search_term: &Option<SearchTerm>,
    ) -> color_eyre::Result<()> {
        self.tree_items = Vec::new();
        self.found_items = Vec::new();

        for item in config.feed_list.iter() {
            use FeedListContentIdentifier::*;
            match item {
                Feeds(item_type) => {
                    self.add_feeds_item(config, model_data, item_type, search_term)?
                }
                Categories(item_type) => {
                    self.add_categories_item(config, model_data, item_type, search_term)?
                }
                Tags(item_type) => {
                    self.add_tags_item(config, model_data, item_type, search_term)
                        .await?
                }
                Query(labeled_query) => self.add_query_item(config, labeled_query, search_term),
            }
        }

        Ok(())
    }

    fn add_query_item(
        &mut self,
        config: &Config,
        labeled_query: &LabeledQuery,
        search_term: &Option<SearchTerm>,
    ) {
        // queries
        let query_item = FeedListItem::Query(Box::new(labeled_query.clone()));
        let mut query_item_text = query_item.to_text(config, None, None);

        if let Some(search_term) = search_term.as_ref()
            && search_term.test_text(&query_item_text)
        {
            query_item_text.style = config.theme.patch_highlighted(&query_item_text.style);
            self.found_items.push(query_item.to_owned());
        }

        self.tree_items
            .push(TreeItem::new_leaf(query_item, query_item_text));
    }

    fn include_feed_or_category(
        &self,
        model_data: &FeedListModelData,
        feed_or_category: &FeedOrCategory,
    ) -> bool {
        match self.scope {
            ArticleScope::All => true,
            ArticleScope::Unread => model_data
                .unread_count_for_feed_or_category()
                .get(feed_or_category)
                .map(|count| *count > 0)
                .unwrap_or(false),
            ArticleScope::Marked => model_data
                .marked_count_for_feed_or_category()
                .get(feed_or_category)
                .map(|count| *count > 0)
                .unwrap_or(false),
        }
    }

    fn add_categories_item(
        &mut self,
        config: &Config,
        model_data: &FeedListModelData,
        item_type: &FeedListItemType,
        search_term: &Option<SearchTerm>,
    ) -> color_eyre::Result<()> {
        let mut root_items = Vec::new();

        let feeds_or_categories = model_data
            .roots()
            .iter()
            .filter(|feed_or_category| self.include_feed_or_category(model_data, feed_or_category))
            .collect::<Vec<&FeedOrCategory>>();

        for root in feeds_or_categories {
            match root {
                FeedOrCategory::Category(category_id) => {
                    if let Some(category) = model_data.category_map().get(category_id) {
                        root_items.push(self.map_category_to_tree_item(
                            config,
                            category,
                            model_data,
                            search_term,
                        ))
                    }
                }

                FeedOrCategory::Feed(feed_id) => {
                    if let Some(feed) = model_data.feed_map().get(feed_id) {
                        root_items.push(self.map_feed_to_tree_item(
                            config,
                            feed,
                            model_data,
                            search_term,
                        ))
                    }
                }
            }
        }

        match item_type {
            FeedListItemType::Tree => {
                let categories_item = FeedListItem::Categories;
                let mut categories_text = categories_item.to_text(config, None, None);
                if let Some(search_term) = search_term.as_ref()
                    && search_term.test_text(&categories_text)
                {
                    categories_text.style = config.theme.patch_highlighted(&categories_text.style);
                    self.found_items.push(categories_item.to_owned());
                }

                self.tree_items
                    .push(TreeItem::new(categories_item, categories_text, root_items)?);
            }
            FeedListItemType::List => {
                self.tree_items.append(&mut root_items);
            }
        }
        Ok(())
    }

    async fn add_tags_item(
        &mut self,
        config: &Config,
        model_data: &FeedListModelData,
        item_type: &FeedListItemType,
        search_term: &Option<SearchTerm>,
    ) -> Result<(), color_eyre::eyre::Error> {
        if model_data
            .features()
            .await?
            .contains(PluginCapabilities::SUPPORT_TAGS)
        {
            let mut children = model_data
                .tags()
                .iter()
                .filter(|tag| {
                    !(matches!(self.scope, ArticleScope::Unread)
                        && model_data
                            .unread_count_for_tag()
                            .get(&tag.tag_id)
                            .map(|count| *count == 0i64)
                            .unwrap_or(false))
                })
                .map(|tag| self.gen_tag_item(config, model_data, tag.to_owned()))
                .collect::<Vec<TreeItem<_>>>();

            match item_type {
                FeedListItemType::List => self.tree_items.append(&mut children),
                FeedListItemType::Tree => {
                    let tags_item = FeedListItem::Tags;
                    let mut tag_item_text = tags_item.to_text(config, None, None);
                    if let Some(search_term) = search_term.as_ref()
                        && search_term.test_text(&tag_item_text)
                    {
                        tag_item_text.style = config.theme.patch_highlighted(&tag_item_text.style);
                        self.found_items.push(tags_item.to_owned());
                    }

                    let tags_tree_item = TreeItem::new(tags_item, tag_item_text, children)?;
                    self.tree_items.push(tags_tree_item);
                }
            }
        }
        Ok(())
    }

    fn add_feeds_item(
        &mut self,
        config: &Config,
        model_data: &FeedListModelData,
        item_type: &FeedListItemType,
        search_term: &Option<SearchTerm>,
    ) -> Result<(), color_eyre::eyre::Error> {
        let feeds = model_data
            .feeds()
            .iter()
            .filter(|feed| {
                self.include_feed_or_category(
                    model_data,
                    &FeedOrCategory::Feed(feed.feed_id.to_owned()),
                )
            })
            .collect::<Vec<&Feed>>();
        let mut children = feeds
            .into_iter()
            .map(|feed| self.map_feed_to_tree_item(config, feed, model_data, search_term))
            .collect();

        match item_type {
            FeedListItemType::List => self.tree_items.append(&mut children),
            FeedListItemType::Tree => self.tree_items.push(TreeItem::new(
                FeedListItem::All,
                FeedListItem::All.to_text(config, Some(*model_data.unread_count_all()), None),
                children,
            )?),
        }
        Ok(())
    }

    fn map_feed_to_tree_item<'a>(
        &mut self,
        config: &Config,
        feed: &Feed,
        model_data: &FeedListModelData,
        search_term: &Option<SearchTerm>,
    ) -> TreeItem<'a, FeedListItem> {
        let identifier = FeedListItem::Feed(Box::new(feed.clone()));
        let mut identifier_text = identifier.to_text(
            config,
            Some(
                model_data
                    .unread_count_for_feed_or_category()
                    .get(&FeedOrCategory::Feed(feed.feed_id.clone()))
                    .copied()
                    .unwrap_or(0),
            ),
            Some(
                model_data
                    .marked_count_for_feed_or_category()
                    .get(&FeedOrCategory::Feed(feed.feed_id.clone()))
                    .copied()
                    .unwrap_or(0),
            ),
        );

        if let Some(UnifiedMapping::Feed(feed_mapping)) = self.yanked_unified_mapping()
            && feed_mapping.feed_id == feed.feed_id
        {
            identifier_text = identifier_text.style(config.theme.yanked());
        }

        if let Some(search_term) = search_term.as_ref()
            && search_term.test_text(&identifier_text)
        {
            identifier_text.style = config.theme.patch_highlighted(&identifier_text.style);
            self.found_items.push(identifier.to_owned());
        }

        TreeItem::new_leaf(identifier, identifier_text)
    }

    fn map_category_to_tree_item<'a>(
        &mut self,
        config: &Config,
        category: &Category,
        model_data: &FeedListModelData,
        search_term: &Option<SearchTerm>,
    ) -> TreeItem<'a, FeedListItem> {
        let mut children: Vec<TreeItem<'a, FeedListItem>> = Vec::new();

        if let Some(child_categories) = model_data.category_tree().get(&category.category_id) {
            let child_categories = child_categories
                .iter()
                .filter(|feed_or_category| {
                    self.include_feed_or_category(model_data, feed_or_category)
                })
                .collect::<Vec<&FeedOrCategory>>();

            for child in child_categories {
                children.push(match child {
                    FeedOrCategory::Category(category_id) => {
                        let child_category = model_data.category_map().get(category_id).unwrap();
                        self.map_category_to_tree_item(
                            config,
                            child_category,
                            model_data,
                            search_term,
                        )
                    }

                    FeedOrCategory::Feed(feed_id) => {
                        let feed = model_data.feed_map().get(feed_id).unwrap();
                        self.map_feed_to_tree_item(config, feed, model_data, search_term)
                    }
                });
            }
        }

        let identifier = FeedListItem::Category(Box::new(category.clone()));
        let unread_category = model_data
            .unread_count_for_feed_or_category()
            .get(&category.category_id.clone().into())
            .copied();
        let marked_category = model_data
            .marked_count_for_feed_or_category()
            .get(&category.category_id.clone().into())
            .copied();
        let mut identifier_text = identifier.to_text(config, unread_category, marked_category);

        if let Some(search_term) = search_term.as_ref()
            && search_term.test_text(&identifier_text)
        {
            identifier_text.style = config.theme.patch_highlighted(&identifier_text.style);
            self.found_items.push(identifier.to_owned());
        }

        if let Some(UnifiedMapping::Category(category_mapping)) = self.yanked_unified_mapping()
            && category_mapping.category_id == category.category_id
        {
            identifier_text = identifier_text.style(config.theme.yanked());
        }

        TreeItem::new(identifier, identifier_text, children).unwrap()
    }

    fn gen_tag_item(
        &self,
        config: &Config,
        model_data: &FeedListModelData,
        tag: news_flash::models::Tag,
    ) -> TreeItem<'static, FeedListItem> {
        let count = model_data
            .unread_count_for_tag()
            .get(&tag.tag_id)
            .copied()
            .unwrap_or(0);
        let tag_item = FeedListItem::Tag(Box::new(tag));
        let tag_item_text = tag_item.to_text(config, Some(count), None);
        TreeItem::new_leaf(tag_item, tag_item_text)
    }

    pub(super) fn take_yanked_unified_mapping(&mut self) -> Option<UnifiedMapping> {
        self.yanked_unified_mapping.take()
    }

    pub(super) fn set_yanked_unified_mapping(&mut self, unified_mapping: Option<UnifiedMapping>) {
        self.yanked_unified_mapping = unified_mapping;
    }

    pub(super) fn yank_feed_or_category(
        &mut self,
        feed_or_category: FeedOrCategory,
        model_data: &FeedListModelData,
    ) {
        info!("yanked {:?}", feed_or_category);
        // self.yanked_feed_or_category = Some(feed_or_category);

        use FeedOrCategory::*;
        self.yanked_unified_mapping = match feed_or_category {
            Feed(feed_id) => {
                let feed_mapping = match model_data.feed_mapping_for_feed().get(&feed_id) {
                    Some(feed_mapping) => feed_mapping.to_owned(),
                    None => FeedMapping {
                        feed_id: feed_id.to_owned(),
                        category_id: (*NEWSFLASH_TOPLEVEL).to_owned(),
                        sort_index: None,
                    },
                };
                Some(UnifiedMapping::Feed(feed_mapping))
            }
            Category(category_id) => model_data
                .category_mapping_for_category()
                .get(&category_id)
                .map(|mapping| UnifiedMapping::Category(mapping.to_owned())),
        };
    }

    pub(super) fn get_selection_with_index(
        &self,
        selection: &[FeedListItem],
    ) -> Vec<(usize, FeedListItem)> {
        selection
            .iter()
            .scan(self.tree_items.as_slice(), |items, id| {
                items
                    .iter()
                    .enumerate()
                    .find(|(_, item)| item.identifier() == id)
                    .map(|(index, next)| {
                        *items = next.children();
                        (index, next.identifier().to_owned())
                    })
                    .or(None)
            })
            .collect::<Vec<(usize, FeedListItem)>>()
    }

    pub(super) fn ensure_sensible_selection(
        &mut self,
        selected_before: &[(usize, FeedListItem)],
    ) -> bool {
        let sensible_selection = selected_before
            .iter()
            .scan(
                (false, self.tree_items.as_slice()),
                |(last, items), (index, id)| {
                    items
                        .iter()
                        .enumerate()
                        .find(|(_, item)| item.identifier() == id)
                        .map(|(_, next)| {
                            *items = next.children();
                            next.identifier().to_owned()
                        })
                        .or_else(|| {
                            if *last {
                                None
                            } else {
                                *last = true;
                                items
                                    .get(*index) // try index
                                    .or(items.get((*index).saturating_sub(1))) // or the one before
                                    .or(items.get((*index).saturating_add(1))) // or the one after
                                    .map(|item| item.identifier().to_owned())
                            }
                        })
                },
            )
            .collect::<Vec<FeedListItem>>();

        if !sensible_selection.is_empty() {
            self.tree_state.select(sensible_selection)
        } else {
            self.tree_state.select(
                self.tree_items
                    .first()
                    .map(|first_item| vec![first_item.identifier().to_owned()])
                    .unwrap_or_default(),
            )
        }
    }

    fn build_title<'a>(&self, config: &Config) -> Line<'a> {
        let mut title = Line::styled("", config.theme.header());
        let spans = &mut title.spans;
        for scope in ArticleScope::iter() {
            let style = if scope == self.scope {
                config.theme.header()
            } else {
                config.theme.inactive()
            };
            spans.push(" ".into());
            spans.push(Span::styled(scope.to_icon(config).to_string(), style));
        }
        spans.push(" ".into());
        title
    }
}
