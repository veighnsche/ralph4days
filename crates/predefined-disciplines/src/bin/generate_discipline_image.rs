use predefined_disciplines::{
    get_disciplines_for_stack, get_global_image_prompts, get_stack_metadata, DISCIPLINE_WORKFLOW,
    DISCIPLINE_WORKFLOW_TEST,
};
use ralph_external::DisciplinePrompts;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut test_mode = false;
    let filtered: Vec<&str> = args
        .iter()
        .skip(1)
        .filter(|a| {
            if *a == "--test" {
                test_mode = true;
                false
            } else {
                true
            }
        })
        .map(String::as_str)
        .collect();

    let (stack_num, disc_num) = if filtered.len() == 2 {
        let s: u8 = filtered[0]
            .parse()
            .expect("stack must be a zero-padded number (01-04)");
        let d: usize = filtered[1]
            .parse()
            .expect("discipline must be a zero-padded number (00-07)");
        (s, d)
    } else {
        eprintln!("Usage: generate-discipline-image [--test] <stack> <discipline>");
        eprintln!("Example: generate-discipline-image 02 00");
        eprintln!("         generate-discipline-image --test 01 03");
        std::process::exit(1);
    };

    let workflow = if test_mode {
        eprintln!("Using test workflow (1 step)");
        DISCIPLINE_WORKFLOW_TEST
    } else {
        DISCIPLINE_WORKFLOW
    };

    let global = get_global_image_prompts();

    let stack = get_stack_metadata(stack_num).unwrap_or_else(|| {
        eprintln!("Unknown stack: {stack_num:02}");
        std::process::exit(1);
    });

    let stack_prompt = stack.image_prompt.unwrap_or_else(|| {
        eprintln!("Stack {stack_num:02} has no image_prompt defined in ABOUT.yaml");
        std::process::exit(1);
    });

    let disciplines = get_disciplines_for_stack(stack_num);
    let discipline = disciplines.get(disc_num).unwrap_or_else(|| {
        eprintln!(
            "Discipline {disc_num:02} not found in stack {stack_num:02} (has {} disciplines: 00-{:02})",
            disciplines.len(),
            disciplines.len().saturating_sub(1)
        );
        std::process::exit(1);
    });

    let disc_prompt = discipline.image_prompt.as_ref().unwrap_or_else(|| {
        eprintln!(
            "Discipline {:02}_{} has no image_prompt defined",
            disc_num, discipline.name
        );
        std::process::exit(1);
    });

    let prompts = DisciplinePrompts {
        positive_global: global.global.positive.clone(),
        negative_global: global.global.negative.clone(),
        positive_stack: stack_prompt.positive,
        negative_stack: stack_prompt.negative,
        positive_discipline: disc_prompt.positive.clone(),
        negative_discipline: disc_prompt.negative.clone(),
    };

    eprintln!(
        "Generating portrait for: {:02}_{} (stack {stack_num:02})",
        disc_num, discipline.display_name
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
        workflow,
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
            let stack_slug = match stack_num {
                1 => "generic",
                2 => "desktop",
                3 => "saas",
                4 => "mobile",
                _ => "unknown",
            };
            let stack_dir = format!(
                "crates/predefined-disciplines/src/defaults/disciplines/{stack_num:02}_{stack_slug}/images",
            );
            std::fs::create_dir_all(&stack_dir).expect("Failed to create images directory");

            let suffix = if test_mode { "_test" } else { "" };
            let output_path = format!(
                "{stack_dir}/{:02}_{}{suffix}.png",
                disc_num, discipline.name
            );
            std::fs::write(&output_path, &image_bytes).expect("Failed to write output file");
            eprintln!("Saved: {output_path} ({} bytes)", image_bytes.len());
        }
        Err(e) => {
            eprintln!("Generation failed: {e}");
            std::process::exit(1);
        }
    }
}
