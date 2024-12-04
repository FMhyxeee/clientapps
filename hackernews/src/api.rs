use anyhow::Result;
use futures::future::join_all;

use crate::{Comment, StoryData, StoryItem};

const MAX_STORIES: usize = 50;

pub async fn get_top_stories(n: usize) -> Result<Vec<StoryItem>> {
    let n = n.min(MAX_STORIES);

    let url = "https://hacker-news.firebaseio.com/v0/topstories.json";
    let ids: Vec<i64> = reqwest::get(url).await?.json().await?;

    let stories = ids.into_iter().take(n).map(get_story_item_by_id);

    let stories = join_all(stories)
        .await
        .into_iter()
        .filter_map(|item| item.ok())
        .collect::<Vec<_>>();

    Ok(stories)
}

pub async fn get_story_item_by_id(id: i64) -> Result<StoryItem> {
    let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
    let item: StoryItem = reqwest::get(url).await?.json().await?;

    Ok(item)
}

// only retrieve top level comments
pub async fn get_story_comments(item: StoryItem) -> Result<StoryData> {
    let comments_futures = item.kids.iter().map(|id| get_comment_by_id(*id));

    let comments = join_all(comments_futures)
        .await
        .into_iter()
        .filter_map(|item| item.ok())
        .collect::<Vec<_>>();

    let story_data = StoryData { item, comments };

    Ok(story_data)
}

pub async fn get_comment_by_id(id: i64) -> Result<Comment> {
    let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
    let comment: Comment = reqwest::get(url).await?.json().await?;

    Ok(comment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn get_top_stories_should_work() {
        let stories = get_top_stories(3).await.unwrap();
        assert_eq!(stories.len(), 3);
    }

    #[ignore]
    #[tokio::test]
    async fn get_comment_by_id_should_work() {
        let story = get_top_stories(1).await.unwrap();
        let id = story[0].kids[0];

        let comment = get_comment_by_id(id).await.unwrap();
        assert_eq!(comment.id, id);
    }
}
