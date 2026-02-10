use std::str::FromStr;

use news_flash::models::Url;
use ratatui::style::{Color, ParseColorError};
use serde::Deserialize;

use crate::prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum CommandParseError {
    #[error("expecting command")]
    CommandExpected,

    #[error("expecting command name")]
    CommandNameExpected(#[from] strum::ParseError),

    #[error("expecting tag")]
    TagExpected,

    #[error("expecting article scope")]
    ArticleScopeExpected,

    #[error("expecting target or article scope")]
    TargetOrArticleScopeExpected,

    #[error("expecting color")]
    ColorExpected(#[from] ParseColorError),

    #[error("expecting URL")]
    URLExpected(#[from] url::ParseError),

    #[error("expecting panel")]
    PanelExpected,

    #[error("expecting position")]
    PositionExpected,

    #[error("expecting article search query")]
    ArticleQueryExpected(#[from] QueryParseError),

    #[error("expecting share target")]
    ShareTargetExpected,

    #[error("expecting file path")]
    FilePathExpected,

    #[error("sort order expected")]
    SortOrderExpected(#[from] SortOrderParseError),

    #[error("expecting `NOW` as parameter for confirmation")]
    ConfirmationExpected,

    #[error("search term expected (quoted string, regex or single word)")]
    SearchTermExpected,

    #[error("expecting a word")]
    WordExpected(String),

    #[error("expecting something")]
    SomethingExpected(String),

    #[error("unexpected")]
    NothingExcepted(String),
}

fn split_off_first(s: &str) -> (String, Option<String>) {
    let trimmed = s.trim();
    let end_pos = trimmed.find(" ");

    let first = match end_pos {
        Some(pos) => &trimmed[..pos],
        None => trimmed,
    };

    let args = end_pos
        .map(|pos| (trimmed[pos + 1..]).to_owned())
        .to_owned();

    (first.trim().to_owned(), args)
}

fn expect_word(s: &mut Option<String>, to_expect: &str) -> Result<String, CommandParseError> {
    let Some(args) = s.as_mut() else {
        return Err(CommandParseError::WordExpected(to_expect.to_owned()));
    };

    let (word, tail) = split_off_first(args.as_str());

    *s = tail;

    Ok(word)
}

fn expect_something(s: Option<String>, to_expect: &str) -> Result<String, CommandParseError> {
    s.ok_or(CommandParseError::SomethingExpected(to_expect.to_owned()))
}

fn expect_from_str<T: FromStr>(
    s: &mut Option<String>,
    to_expect: &str,
) -> Result<T, CommandParseError>
where
    T::Err: Into<CommandParseError>,
{
    let word = expect_word(s, to_expect);
    T::from_str(word.as_deref().unwrap_or_default()).map_err(|e| e.into())
}

fn expect_nothing(s: Option<String>) -> Result<(), CommandParseError> {
    match s {
        Some(s) => Err(CommandParseError::NothingExcepted(s)),
        None => Ok(()),
    }
}

impl Command {
    pub fn parse(s: &str, eager: bool) -> Result<Self, CommandParseError> {
        use CommandParseError as E;

        let mut args = if s.is_empty() {
            None
        } else {
            Some(s.to_owned())
        };

        let command: Command = expect_from_str(&mut args, "expecting command")?;

        use Command as C;
        Ok(match command {
            C::CommandConfirm(_) => {
                let Some(args) = args else {
                    return Err(E::CommandExpected);
                };
                C::CommandConfirm(Box::new(Command::parse(&args, eager)?))
            }

            C::In(..) => {
                let panel: Panel =
                    expect_from_str(&mut args, "expecting panel: feeds, articles, content")?;

                let Some(args) = args else {
                    return Err(E::CommandExpected);
                };

                C::In(panel, Box::new(Command::parse(&args, eager)?))
            }

            C::PanelFocus(_) => {
                let panel: Panel =
                    expect_from_str(&mut args, "expecting panel: feeds, articles, content")?;
                C::PanelFocus(panel)
            }

            C::ActionSetRead(..) => {
                let old_args = args.clone();
                let word = expect_word(&mut args, "target or scope")
                    .map_err(|_| E::TargetOrArticleScopeExpected);

                if eager && let Err(error) = word {
                    return Err(error);
                };

                let target = word
                    .map(|word| match ActionTarget::from_str(word.as_str()) {
                        Ok(target) => target,
                        Err(_) => {
                            args = old_args; // restore old version of args
                            ActionTarget::Current
                        }
                    })
                    .unwrap_or(ActionTarget::Current);

                C::ActionSetRead(target, ActionScope::from_option_string(args.as_deref())?)
            }

            C::ActionSetUnread(..) => {
                C::ActionSetUnread(ActionScope::from_option_string(args.as_deref())?)
            }
            C::ActionOpenInBrowser(..) => {
                C::ActionOpenInBrowser(ActionScope::from_option_string(args.as_deref())?)
            }
            C::ActionSetMarked(..) => {
                C::ActionSetMarked(ActionScope::from_option_string(args.as_deref())?)
            }
            C::ActionSetUnmarked(..) => {
                C::ActionSetUnmarked(ActionScope::from_option_string(args.as_deref())?)
            }
            tag_command @ (C::ActionTagArticles(..) | C::ActionUntagArticles(..)) => {
                let tag =
                    expect_word(&mut args, "expecting tag name").map_err(|_| E::TagExpected)?;

                match tag_command {
                    C::ActionTagArticles(..) => {
                        C::ActionTagArticles(ActionScope::from_option_string(args.as_deref())?, tag)
                    }
                    C::ActionUntagArticles(..) => C::ActionUntagArticles(
                        ActionScope::from_option_string(args.as_deref())?,
                        tag,
                    ),
                    _ => unreachable!(),
                }
            }

            C::FeedListFeedAdd(..) => {
                let url = Url::new(expect_from_str::<reqwest::Url>(
                    &mut args,
                    "expecting feed URL",
                )?);
                let name = args;
                C::FeedListFeedAdd(Some(url), name)
            }

            C::FeedListFeedChangeUrl(..) => {
                let url = Url::new(expect_from_str::<reqwest::Url>(
                    &mut args,
                    "expecting feed URL",
                )?);
                expect_nothing(args)?;
                C::FeedListFeedChangeUrl(Some(url))
            }

            C::FeedListPasteFeedOrCategory(..) => {
                let position =
                    expect_from_str::<PastePosition>(&mut args, "expecting paste position")
                        .map_err(|_| E::PositionExpected)?;
                expect_nothing(args)?;
                C::FeedListPasteFeedOrCategory(position)
            }

            C::FeedListCategoryAdd(..) => {
                C::FeedListCategoryAdd(expect_something(args, "expecting category name")?)
            }

            C::FeedListRenameEntity(..) => {
                C::FeedListRenameEntity(expect_something(args, "expecting new name")?)
            }

            C::FeedListTagChangeColor(..) => {
                let color: Color = expect_from_str(&mut args, "expecting tag color")
                    .map_err(|_| E::ColorExpected(ParseColorError))?;
                expect_nothing(args)?;
                C::FeedListTagChangeColor(color)
            }

            C::TagAdd(..) => {
                let tag_title = expect_word(&mut args, "expecting tag name")?;
                let color: Option<Color> = match args {
                    None => {
                        if eager {
                            return Err(E::ColorExpected(ParseColorError));
                        } else {
                            None
                        }
                    }
                    _ => Some(expect_from_str(&mut args, "expecting tag color")?),
                };
                expect_nothing(args)?;
                C::TagAdd(tag_title, color)
            }

            C::FeedListExpandCategories(_) => C::FeedListExpandCategories(
                expect_from_str(&mut args, "article scope expected")
                    .map_err(|_| E::ArticleScopeExpected)?,
            ),

            C::Show(..) => {
                let old_args = args.clone();
                let word = expect_word(&mut args, "target or scope")
                    .map_err(|_| E::TargetOrArticleScopeExpected);

                if eager && let Err(error) = word {
                    return Err(error);
                };

                let target = word
                    .map(|word| match ActionTarget::from_str(word.as_str()) {
                        Ok(target) => target,
                        Err(_) => {
                            args = old_args; // restore old version of args
                            ActionTarget::Current
                        }
                    })
                    .unwrap_or(ActionTarget::Current);

                C::Show(
                    target,
                    expect_from_str::<ArticleScope>(&mut args, "expecting article scope")
                        .map_err(|_| E::ArticleScopeExpected)?,
                )
            }

            C::ArticleListSearch(..) => C::ArticleListSearch(ArticleQuery::from_str(
                expect_something(args, "expecting article query")?.as_str(),
            )?),

            C::Search(..) => C::Search(Some(
                expect_from_str::<SearchTerm>(&mut args, "expecting search term")
                    .map_err(|_| E::SearchTermExpected)?,
            )),

            C::ArticleListFilterSet(..) => C::ArticleListFilterSet(ArticleQuery::from_str(
                expect_something(args, "expecting article query")
                    .map_err(|_| {
                        E::ArticleQueryExpected(QueryParseError::KeyOrWordExpected(
                            0,
                            "".to_owned(),
                        ))
                    })?
                    .as_str(),
            )?),

            C::ArticleListQuery(..) => C::ArticleListQuery(ArticleQuery::from_str(
                expect_something(args, "expecting article query")
                    .map_err(|_| {
                        E::ArticleQueryExpected(QueryParseError::KeyOrWordExpected(
                            0,
                            "".to_owned(),
                        ))
                    })?
                    .as_str(),
            )?),

            C::ArticleListSort(..) => C::ArticleListSort(SortOrder::from_str(
                expect_something(args, "expecting sort order")
                    .map_err(|_| {
                        E::SortOrderExpected(SortOrderParseError::OrderDirectionOrKeyExpected(
                            0,
                            "".to_owned(),
                        ))
                    })?
                    .as_str(),
            )?),

            C::ArticleShare(..) => C::ArticleShare(
                expect_word(&mut args, "expecting share target")
                    .map_err(|_| E::ShareTargetExpected)?,
            ),

            cmd @ (C::ExportOpml(..) | C::ImportOpml(..)) => {
                let path = expect_something(args, "expecting file path")
                    .map_err(|_| E::FilePathExpected)?;

                match cmd {
                    C::ExportOpml(..) => C::ExportOpml(path),
                    C::ImportOpml(..) => C::ImportOpml(path),
                    _ => unreachable!(),
                }
            }

            C::Logout(..) => {
                let word = expect_word(
                    &mut args,
                    "expecting confirmation `NOW` if you are really sure",
                )
                .map_err(|_| E::ConfirmationExpected)?;
                expect_nothing(args)?;
                C::Logout(word)
            }

            C::CommandLineOpen(..) => C::CommandLineOpen(args),

            command_without_args => command_without_args,
        })
    }
}

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Command::parse(&s, false).map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}
