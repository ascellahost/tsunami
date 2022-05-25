use criterion::{criterion_group, criterion_main, Criterion};
use futures::future::join_all;

use reqwest::multipart::{Form, Part};
use tokio::fs::read;
use tsunami::CLIENT;

fn bench_requests(c: &mut Criterion) {
    // We are using System here, since Runtime requires preinitialized tokio
    // Maybe add to actix_rt docs
    let rt = actix_web::rt::System::new();

    // let srv = rt.block_on(async {
    //     let data = web::Data::new(Mutex::new(ImageCache::new()));
    //     actix_test::start(move || {
    //         let spec = DefaultApiRaw {
    //             info: Info {
    //                 version: "2.0".into(),
    //                 title: "Ascella Image uploader".into(),
    //                 description:
    // Some(include_str!("../api_description.md").into()),
    // contact: Some(Contact {                     name: Some("Tricked".into()),
    //                     url: Some("https://tricked.pro".into()),
    //                     email: Some("tricked@tricked.pro".into()),
    //                 }),
    //                 license: Some(License {
    //                     name: Some("AGPL-3.0".into()),
    //                     url: Some("https://github.com/ascellahost/tsunami/blob/master/LICENSE".into()),
    //                 }),
    //                 ..Default::default()
    //             },
    //             tags: vec![
    //                 Tag {
    //                     name: "Images".to_string(),
    //                     description: Some("Stuff related to images".to_string()),
    //                     external_docs: None,
    //                 },
    //                 Tag {
    //                     name: "Dashboard".to_string(),
    //                     description: Some(
    //                         "Stuff related to the
    // Dashboard"
    //                             .to_string(),
    //                     ),
    //                     external_docs: None,
    //                 },
    //                 Tag {
    //                     name: "Etc".to_string(),
    //                     description: Some(
    //                         "Stuff not related to the
    // above"
    //                             .to_string(),
    //                     ),
    //                     external_docs: None,
    //                 },
    //             ],
    //             host: Some("ascella.wtf".into()),
    //             ..DefaultApiRaw::default()
    //         };

    //         let cors = Cors::default()
    //             .allow_any_origin()
    //             .allow_any_header()
    //             .allow_any_method()
    //             .max_age(3600);

    //         App::new()
    //             .service(openapi_3::get)
    //             .wrap_api_with_spec(spec)
    //             .app_data(data.clone())
    //             .wrap(cors)
    //             .wrap(Governor::new(
    //                 &GovernorConfigBuilder::default()
    //                     .per_second(60)
    //                     .burst_size(30)
    //                     .finish()
    //                     .unwrap(),
    //             ))
    //             .wrap(Governor::new(
    //                 &GovernorConfigBuilder::default()
    //                     .per_second(3600)
    //                     .burst_size(128)
    //                     .finish()
    //                     .unwrap(),
    //             ))
    //             .wrap(middleware::Logger::default())
    //             .with_json_spec_at("/v2/ascella/spec/v2")
    //             .service(web::scope("/v2/ascella").configure(set_endpoints))
    //             .default_service(web::to(|| async {
    //                 Error::Four04 {
    //                     message: "Path not found.".to_owned(),
    //                 }
    //                 .error_response()
    //             }))
    //             .build()
    //     })
    // });

    // let url = srv.url("/v2/ascella/upload");

    let url = "http://localhost:7878/v2/ascella/upload".to_owned();

    c.bench_function("get_body_async_burst", move |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let image = read("/home/tricked/coding/Ascella/tsunami/test_image.png")
                    .await
                    .unwrap();

                let start = std::time::Instant::now();
                // benchmark body

                let burst = (0..iters).map(|_| {
                    let form = Form::new().part("file", Part::bytes(image.clone()).mime_str("image/png").unwrap());
                    CLIENT
                        .post(url.clone())
                        .header("Authorization", "f9df8137-f9a6-4674-8abc-128a7f74ef03")
                        .multipart(form)
                        .send()
                });
                let resps = join_all(burst).await;

                let elapsed = start.elapsed();

                // if there are failed requests that might be an issue
                let failed = resps.iter().filter(|r| r.is_err()).count();
                if failed > 0 {
                    eprintln!("failed {} requests (might be bench timeout)", failed);
                };

                elapsed
            })
        })
    });
}
criterion_group!(benches, bench_requests);
criterion_main!(benches);
