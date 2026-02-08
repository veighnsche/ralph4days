use predefined_disciplines::{
    get_disciplines_for_stack, get_global_image_prompts, get_stack_metadata, DISCIPLINE_WORKFLOW,
};
use ralph_external::DisciplinePrompts;

struct Args {
    stack: u8,
    discipline: usize,
    steps: u32,
    ratio_w: f64,
    ratio_h: f64,
    megapixels: f64,
}

fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut steps = 28u32;
    let mut ratio_w = 1.0f64;
    let mut ratio_h = 1.0f64;
    let mut megapixels = 1.5f64;
    let mut positional = Vec::new();
    let mut i = 0;

    while i < raw.len() {
        match raw[i].as_str() {
            "--test" => steps = 1,
            "--half" => steps = 14,
            "--ratio-portrait" => {
                ratio_w = 6.0;
                ratio_h = 19.0;
            }
            "--ratio" => {
                ratio_w = raw.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                    eprintln!("--ratio requires two numbers: --ratio W H");
                    std::process::exit(1);
                });
                ratio_h = raw.get(i + 2).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                    eprintln!("--ratio requires two numbers: --ratio W H");
                    std::process::exit(1);
                });
                i += 2;
            }
            "--mp" => {
                megapixels = raw.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                    eprintln!("--mp requires a number: --mp 1.5");
                    std::process::exit(1);
                });
                i += 1;
            }
            other => positional.push(other.to_owned()),
        }
        i += 1;
    }

    if positional.len() != 2 {
        eprintln!("Usage: generate-discipline-image <stack> <discipline> [flags]");
        eprintln!();
        eprintln!("Flags:");
        eprintln!("  --test             1 step (pipeline test)");
        eprintln!("  --half             14 steps (preview)");
        eprintln!("  --ratio W H        aspect ratio (default: 1 1)");
        eprintln!("  --ratio-portrait   shorthand for --ratio 6 19");
        eprintln!("  --mp N             megapixels (default: 1.5)");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  generate-discipline-image 02 00");
        eprintln!("  generate-discipline-image 02 00 --half");
        eprintln!("  generate-discipline-image 02 00 --test --ratio-portrait");
        eprintln!("  generate-discipline-image 02 00 --ratio 2 1 --mp 2.0");
        std::process::exit(1);
    }

    let stack: u8 = positional[0]
        .parse()
        .expect("stack must be a number (01-04)");
    let discipline: usize = positional[1]
        .parse()
        .expect("discipline must be a number (00-07)");

    Args {
        stack,
        discipline,
        steps,
        ratio_w,
        ratio_h,
        megapixels,
    }
}

#[tokio::main]
async fn main() {
    let args = parse_args();
    let (width, height) =
        ralph_external::compute_dimensions(args.ratio_w, args.ratio_h, args.megapixels);

    eprintln!(
        "Settings: {} steps, {}x{} ({:.1}MP, ratio {}:{})",
        args.steps, width, height, args.megapixels, args.ratio_w, args.ratio_h
    );

    let mut workflow: std::collections::HashMap<String, ralph_external::WorkflowNode> =
        serde_json::from_str(DISCIPLINE_WORKFLOW).expect("embedded workflow is valid JSON");
    ralph_external::set_steps(&mut workflow, args.steps);
    ralph_external::set_dimensions(&mut workflow, width, height);
    let global = get_global_image_prompts();

    let stack = get_stack_metadata(args.stack).unwrap_or_else(|| {
        eprintln!("Unknown stack: {:02}", args.stack);
        std::process::exit(1);
    });

    let stack_prompt = stack.image_prompt.unwrap_or_else(|| {
        eprintln!(
            "Stack {:02} has no image_prompt defined in ABOUT.yaml",
            args.stack
        );
        std::process::exit(1);
    });

    let disciplines = get_disciplines_for_stack(args.stack);
    let discipline = disciplines.get(args.discipline).unwrap_or_else(|| {
        eprintln!(
            "Discipline {:02} not found in stack {:02} (has {} disciplines: 00-{:02})",
            args.discipline,
            args.stack,
            disciplines.len(),
            disciplines.len().saturating_sub(1)
        );
        std::process::exit(1);
    });

    let disc_prompt = discipline.image_prompt.as_ref().unwrap_or_else(|| {
        eprintln!(
            "Discipline {:02}_{} has no image_prompt defined",
            args.discipline, discipline.name
        );
        std::process::exit(1);
    });

    let prompt_txt = format!(
        "positive_global:\n{}\n\nnegative_global:\n{}\n\npositive_stack:\n{}\n\nnegative_stack:\n{}\n\npositive_discipline:\n{}\n\nnegative_discipline:\n{}",
        global.global.positive, global.global.negative,
        stack_prompt.positive, stack_prompt.negative,
        disc_prompt.positive, disc_prompt.negative,
    );

    let prompts = DisciplinePrompts {
        positive_global: global.global.positive,
        negative_global: global.global.negative,
        positive_stack: stack_prompt.positive,
        negative_stack: stack_prompt.negative,
        positive_discipline: disc_prompt.positive.clone(),
        negative_discipline: disc_prompt.negative.clone(),
    };

    eprintln!(
        "Generating portrait for: {:02}_{} (stack {:02})",
        args.discipline, discipline.display_name, args.stack
    );

    let config = ralph_external::ExternalServicesConfig::load()
        .unwrap_or_else(|e| {
            eprintln!("Failed to load config: {e}");
            eprintln!("Using defaults (ComfyUI at localhost:8188)");
            ralph_external::ExternalServicesConfig::default()
        })
        .comfy;

    let status = ralph_external::check_comfy_available(&config).await;
    if !status.available {
        eprintln!(
            "ComfyUI not available: {}",
            status.error.unwrap_or_default()
        );
        std::process::exit(1);
    }

    let result = ralph_external::generate_discipline_portrait_with_progress(
        &config,
        prompts,
        &mut workflow,
        |p| {
            let filled = (p.step as usize * 30) / p.total.max(1) as usize;
            let empty = 30 - filled;
            eprint!(
                "\r  [{}{}] {}/{} steps",
                "█".repeat(filled),
                "░".repeat(empty),
                p.step,
                p.total
            );
        },
    )
    .await;

    eprintln!();

    match result {
        Ok(image_bytes) => {
            let stack_slug = match args.stack {
                1 => "generic",
                2 => "desktop",
                3 => "saas",
                4 => "mobile",
                _ => "unknown",
            };
            let stack_dir = format!(
                "crates/predefined-disciplines/src/defaults/disciplines/{:02}_{stack_slug}/images",
                args.stack,
            );
            std::fs::create_dir_all(&stack_dir).expect("Failed to create images directory");

            let output_path = format!(
                "{stack_dir}/{:02}_{}_{}_{}x{}.png",
                args.discipline, discipline.name, args.steps, width, height
            );
            std::fs::write(&output_path, &image_bytes).expect("Failed to write output file");

            let txt_path = output_path.replace(".png", ".txt");
            std::fs::write(&txt_path, &prompt_txt).expect("Failed to write prompt file");

            eprintln!("Saved: {output_path} ({} bytes)", image_bytes.len());
        }
        Err(e) => {
            eprintln!("Generation failed: {e}");
            std::process::exit(1);
        }
    }
}
