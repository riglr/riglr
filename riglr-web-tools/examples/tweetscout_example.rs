//! Example of using the TweetScout API integration for Twitter/X account analysis

use riglr_core::provider::ApplicationContext;
use riglr_web_tools::tweetscout::{
    analyze_account, analyze_social_network, get_account_info, get_account_score,
    get_top_followers, get_top_friends, is_account_credible,
};

async fn analyze_basic_account_info(
    context: &ApplicationContext,
    username: &str,
) {
    println!("\n📊 Account Information:");
    match get_account_info(context, username.to_string()).await {
        Ok(info) => {
            println!("  Name: {}", info.name.unwrap_or_else(|| "N/A".to_string()));
            println!(
                "  Handle: @{}",
                info.screen_name.unwrap_or_else(|| username.to_string())
            );
            println!("  Verified: {}", info.verified.unwrap_or(false));
            println!("  Followers: {}", info.followers_count.unwrap_or(0));
            println!("  Following: {}", info.friends_count.unwrap_or(0));
            println!("  Tweets: {}", info.statuses_count.unwrap_or(0));
            if let Some(desc) = info.description {
                println!("  Bio: {}", desc.chars().take(100).collect::<String>());
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
}

async fn analyze_credibility_score(
    context: &ApplicationContext,
    username: &str,
) {
    println!("\n🎯 Credibility Score:");
    match get_account_score(context, username.to_string()).await {
        Ok(score_resp) => {
            println!("  Score: {:.1}/100", score_resp.score);
            let level = match score_resp.score as i32 {
                80..=100 => "Excellent",
                60..=79 => "Good",
                40..=59 => "Fair",
                20..=39 => "Poor",
                _ => "Very Poor",
            };
            println!("  Level: {}", level);
        }
        Err(e) => println!("  Error: {}", e),
    }
}

async fn perform_quick_credibility_check(
    context: &ApplicationContext,
    username: &str,
) {
    println!("\n✅ Quick Credibility Check:");
    match is_account_credible(
        context,
        username.to_string(),
        Some(50.0), // Threshold of 50/100
    )
    .await
    {
        Ok(check) => {
            println!("  Is Credible: {}", check.is_credible);
            println!("  Score: {:.1}/100", check.score);
            println!("  Verdict: {}", check.verdict);
            println!("  Recommendation: {}", check.recommendation);
        }
        Err(e) => println!("  Error: {}", e),
    }
}

async fn perform_comprehensive_analysis(
    context: &ApplicationContext,
    username: &str,
) {
    println!("\n🔬 Comprehensive Account Analysis:");
    match analyze_account(context, username.to_string()).await {
        Ok(analysis) => {
            println!("  Credibility Score: {:.1}/100", analysis.credibility_score);
            println!("  Score Level: {:?}", analysis.score_level);

            if let Some(ratio) = analysis.follower_ratio {
                println!("  Follower Ratio: {:.2}", ratio);
            }

            println!("\n  📈 Engagement Metrics:");
            println!("    Followers: {}", analysis.engagement.followers);
            println!("    Following: {}", analysis.engagement.following);
            println!("    Posts: {}", analysis.engagement.posts);
            println!(
                "    Engagement Rate: {:.2}%",
                analysis.engagement.engagement_rate
            );
            println!("    Likely Bot: {}", analysis.engagement.likely_bot);
            println!("    Likely Spam: {}", analysis.engagement.likely_spam);

            if !analysis.risk_indicators.is_empty() {
                println!("\n  ⚠️ Risk Indicators:");
                for indicator in &analysis.risk_indicators {
                    println!("    - {}", indicator);
                }
            }

            println!("\n  💡 Assessment:");
            println!("    {}", analysis.assessment);
        }
        Err(e) => println!("  Error: {}", e),
    }
}

async fn display_top_followers(
    context: &ApplicationContext,
    username: &str,
) {
    println!("\n👥 Top Followers (first 5):");
    match get_top_followers(context, username.to_string()).await {
        Ok(followers) => {
            for (i, follower) in followers.iter().take(5).enumerate() {
                println!(
                    "  {}. @{} - {} followers, Score: {:.1}",
                    i + 1,
                    follower.screen_name.as_deref().unwrap_or("unknown"),
                    follower.followers_count.unwrap_or(0),
                    follower.score.unwrap_or(0.0)
                );
            }
            if followers.len() > 5 {
                println!("  ... and {} more", followers.len() - 5);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
}

async fn display_top_friends(
    context: &ApplicationContext,
    username: &str,
) {
    println!("\n👥 Top Friends/Following (first 5):");
    match get_top_friends(context, username.to_string()).await {
        Ok(friends) => {
            for (i, friend) in friends.iter().take(5).enumerate() {
                println!(
                    "  {}. @{} - {} followers, Score: {:.1}",
                    i + 1,
                    friend.screen_name.as_deref().unwrap_or("unknown"),
                    friend.followers_count.unwrap_or(0),
                    friend.score.unwrap_or(0.0)
                );
            }
            if friends.len() > 5 {
                println!("  ... and {} more", friends.len() - 5);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
}

async fn perform_social_network_analysis(
    context: &ApplicationContext,
    username: &str,
) {
    println!("\n🌐 Social Network Analysis:");
    match analyze_social_network(context, username.to_string()).await {
        Ok(network) => {
            println!(
                "  Average Follower Score: {:.1}/100",
                network.avg_follower_score
            );
            println!(
                "  Average Friend Score: {:.1}/100",
                network.avg_friend_score
            );
            println!("  Network Quality: {:?}", network.network_quality);

            if !network.key_influencers.is_empty() {
                println!("\n  🌟 Key Influencers in Network:");
                for influencer in &network.key_influencers {
                    println!("    - {}", influencer);
                }
            }

            println!("\n  Top Followers by Influence:");
            for (i, follower) in network.top_followers.iter().take(3).enumerate() {
                println!(
                    "    {}. @{} ({}) - {} followers",
                    i + 1,
                    follower.username,
                    follower.influence_level,
                    follower.followers
                );
            }

            println!("\n  💡 Network Assessment:");
            println!("    {}", network.assessment);
        }
        Err(e) => println!("  Error: {}", e),
    }
}

async fn test_crypto_account(context: &ApplicationContext) {
    println!("\n{}", "=".repeat(60));
    println!("\n📝 Testing with a crypto account:");
    let crypto_account = "VitalikButerin"; // Ethereum founder

    match analyze_account(context, crypto_account.to_string()).await {
        Ok(analysis) => {
            println!("  Account: @{}", crypto_account);
            println!("  Credibility Score: {:.1}/100", analysis.credibility_score);
            println!("  Score Level: {:?}", analysis.score_level);
            println!("  Likely Bot: {}", analysis.engagement.likely_bot);
            println!("  Assessment: {}", analysis.assessment);
        }
        Err(e) => println!("  Error: {}", e),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create a mock application context (in real usage, this would come from your app)
    let context = ApplicationContext::default();

    // Note: API key is now managed through ApplicationContext
    // Set TWEETSCOUT_API_KEY environment variable before running

    // Example Twitter/X username to analyze (replace with actual username)
    let username = "elonmusk"; // Example high-profile account

    println!("🔍 Analyzing Twitter/X account: @{}", username);
    println!("{}", "=".repeat(60));

    // Run all analyses using extracted helper functions
    analyze_basic_account_info(&context, username).await;
    analyze_credibility_score(&context, username).await;
    perform_quick_credibility_check(&context, username).await;
    perform_comprehensive_analysis(&context, username).await;
    display_top_followers(&context, username).await;
    display_top_friends(&context, username).await;
    perform_social_network_analysis(&context, username).await;
    test_crypto_account(&context).await;

    println!("\n{}", "=".repeat(60));
    println!("✅ TweetScout example completed!");
    println!("\nNote: To use this with real accounts, you'll need to:");
    println!("1. Get an API key from TweetScout.io");
    println!("2. Set TWEETSCOUT_API_KEY environment variable");
    println!("3. Replace the example usernames with accounts you want to analyze");
    println!("\nThis tool is particularly useful for:");
    println!("- Verifying crypto influencer credibility");
    println!("- Detecting bot accounts in token communities");
    println!("- Analyzing social networks of project founders");
    println!("- Due diligence on Twitter/X accounts promoting tokens");

    Ok(())
}
