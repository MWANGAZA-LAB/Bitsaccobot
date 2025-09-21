use bitsacco_whatsapp_bot::{
    config::AppConfig, services::whatsapp::WhatsAppService, types::BotCommand,
};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn create_test_config() -> AppConfig {
    AppConfig {
        whatsapp_access_token: "test_token".to_string(),
        whatsapp_phone_number_id: "test_phone_id".to_string(),
        whatsapp_webhook_verify_token: "test_verify_token".to_string(),
        bitsacco_api_base_url: "https://api.bitsacco.com".to_string(),
        bitsacco_api_token: "test_bitsacco_token".to_string(),
        server_host: "127.0.0.1".to_string(),
        server_port: 8080,
        rust_log: "debug".to_string(),
        rate_limit_requests_per_minute: 60,
        max_message_length: 4096,
        btc_api_base_url: "https://api.coingecko.com/api/v3".to_string(),
        btc_api_key: Some("test_btc_key".to_string()),
    }
}

fn benchmark_bot_command_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("bot_command_parsing");

    group.bench_function("parse_help", |b| {
        b.iter(|| BotCommand::parse(black_box("help")))
    });

    group.bench_function("parse_balance", |b| {
        b.iter(|| BotCommand::parse(black_box("balance")))
    });

    group.bench_function("parse_deposit", |b| {
        b.iter(|| BotCommand::parse(black_box("deposit 100 USD")))
    });

    group.bench_function("parse_withdraw", |b| {
        b.iter(|| BotCommand::parse(black_box("withdraw 50 KES")))
    });

    group.bench_function("parse_transfer", |b| {
        b.iter(|| BotCommand::parse(black_box("transfer 25 USD +254712345678")))
    });

    group.bench_function("parse_unknown", |b| {
        b.iter(|| BotCommand::parse(black_box("unknown command with many words")))
    });

    group.finish();
}

fn benchmark_whatsapp_verification(c: &mut Criterion) {
    let config = create_test_config();
    let whatsapp_service = WhatsAppService::new(&config).unwrap();

    c.bench_function("whatsapp_webhook_verification", |b| {
        b.iter(|| {
            whatsapp_service.verify_webhook(
                black_box("subscribe"),
                black_box("test_verify_token"),
                black_box("challenge123"),
            )
        })
    });
}

fn benchmark_concurrent_requests(c: &mut Criterion) {
    c.bench_function("concurrent_command_parsing", |b| {
        b.iter(|| {
            let commands = vec![
                "help",
                "balance",
                "savings",
                "chama",
                "btc",
                "deposit 100 USD",
                "withdraw 50 KES",
                "transfer 25 USD +254712345678",
            ];

            commands
                .into_iter()
                .map(BotCommand::parse)
                .collect::<Vec<_>>()
        })
    });
}

fn benchmark_message_processing(c: &mut Criterion) {
    c.bench_function("message_processing_simulation", |b| {
        b.iter(|| {
            let messages = vec![
                "help",
                "balance",
                "deposit 100 USD",
                "withdraw 50 KES",
                "transfer 25 USD +254712345678",
                "btc",
                "savings",
                "chama",
            ];

            let results: Vec<_> = messages
                .into_iter()
                .map(|msg| {
                    let command = BotCommand::parse(msg);
                    // Simulate processing time
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    command
                })
                .collect();

            results
        })
    });
}

fn benchmark_high_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("high_throughput");
    group.measurement_time(std::time::Duration::from_secs(10));

    group.bench_function("command_parsing_throughput", |b| {
        b.iter(|| {
            for i in 0..1000 {
                BotCommand::parse(black_box(&format!("command {}", i)));
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_bot_command_parsing,
    benchmark_whatsapp_verification,
    benchmark_concurrent_requests,
    benchmark_message_processing,
    benchmark_high_throughput
);

criterion_main!(benches);
