#!/usr/bin/env python3
import os
import sys
import json
import zipfile
import urllib.request
import shutil

DOWNLOAD_DIR = "/home/user/rMinecraft/downloads"
DATA_DIR = "/home/user/rMinecraft/texture_data"
PACK_DIR = "/home/user/rMinecraft/assets_modern_1_12_2"

JAR_1122_URL = "https://piston-data.mojang.com/v1/objects/0f275bc1547d01fa5f56ba34bdc87d981ee12daf/client.jar"
JAR_LATEST_URL = "https://piston-data.mojang.com/v1/objects/e877b6a07acd633fb3bb475002175cec036e7b87/client.jar"

JAR_1122_PATH = os.path.join(DOWNLOAD_DIR, "client_1.12.2.jar")
JAR_LATEST_PATH = os.path.join(DOWNLOAD_DIR, "client_modern.jar")

def download_file(url, target_path, desc):
    if os.path.exists(target_path) and os.path.getsize(target_path) > 1000000:
        print(f"[{desc}] Already downloaded: {target_path} ({os.path.getsize(target_path)} bytes)")
        return
    print(f"[{desc}] Downloading from {url}...")
    def reporthook(count, block_size, total_size):
        percent = int(count * block_size * 100 / total_size) if total_size > 0 else 0
        sys.stdout.write(f"\r  Progress: {percent}% ({count * block_size // (1024*1024)}MB / {total_size // (1024*1024)}MB)")
        sys.stdout.flush()
    urllib.request.urlretrieve(url, target_path, reporthook)
    print(f"\n[{desc}] Download complete.")

def extract_texture_list(jar_path, version_name):
    print(f"[{version_name}] Reading texture manifest...")
    textures = []
    with zipfile.ZipFile(jar_path, 'r') as zf:
        for name in zf.namelist():
            if name.startswith("assets/minecraft/textures/") and (name.endswith(".png") or name.endswith(".mcmeta")):
                textures.append(name)
    textures.sort()
    json_path = os.path.join(DATA_DIR, f"textures_{version_name}.json")
    with open(json_path, 'w', encoding='utf-8') as f:
        json.dump({"version": version_name, "count": len(textures), "textures": textures}, f, indent=2)
    print(f"[{version_name}] Found {len(textures)} texture files. Saved list to {json_path}")
    return textures

def build_texture_mapping(list_1122, list_modern):
    """
    Map 1.12.2 texture paths to modern texture paths.
    1.12.2 uses `assets/minecraft/textures/blocks/` and `items/`
    Modern uses `assets/minecraft/textures/block/` and `item/`
    Handles 'The Flattening' rename conventions.
    """
    modern_set = set(list_modern)
    modern_by_filename = {}
    for p in list_modern:
        fn = os.path.basename(p)
        modern_by_filename[fn] = p

    # Specific well-known renames from 1.12.2 to 1.13+
    name_rules = {
        # Blocks
        "grass_top.png": "grass_block_top.png",
        "grass_side.png": "grass_block_side.png",
        "grass_side_overlay.png": "grass_block_side_overlay.png",
        "grass_side_snowed.png": "grass_block_snow.png",
        "grass_normal.png": "short_grass.png",
        "stone_granite.png": "granite.png",
        "stone_granite_smooth.png": "polished_granite.png",
        "stone_diorite.png": "diorite.png",
        "stone_diorite_smooth.png": "polished_diorite.png",
        "stone_andesite.png": "andesite.png",
        "stone_andesite_smooth.png": "polished_andesite.png",
        "brick.png": "bricks.png",
        "hardened_clay.png": "terracotta.png",
        "web.png": "cobweb.png",
        "torch_on.png": "torch.png",
        "redstone_torch_on.png": "redstone_torch.png",
        "rail_normal.png": "rail.png",
        "rail_normal_turned.png": "rail_corner.png",
        "rail_golden.png": "powered_rail.png",
        "rail_golden_powered.png": "powered_rail_on.png",
        "rail_activator.png": "activator_rail.png",
        "rail_activator_powered.png": "activator_rail_on.png",
        "rail_detector.png": "detector_rail.png",
        "rail_detector_powered.png": "detector_rail_on.png",
        "cobblestone_mossy.png": "mossy_cobblestone.png",
        "ice_packed.png": "packed_ice.png",
        "reeds.png": "sugar_cane.png",
        "trapdoor.png": "oak_trapdoor.png",
        "door_wood_lower.png": "oak_door_bottom.png",
        "door_wood_upper.png": "oak_door_top.png",
        "door_iron_lower.png": "iron_door_bottom.png",
        "door_iron_upper.png": "iron_door_top.png",
        "slime.png": "slime_block.png",
        "sponge_wet.png": "wet_sponge.png",
        "purpur_pillar.png": "purpur_pillar.png",
        "end_bricks.png": "end_stone_bricks.png",
        "observer_back_lit.png": "observer_back_on.png",
        # Items
        "wood_sword.png": "wooden_sword.png",
        "wood_pickaxe.png": "wooden_pickaxe.png",
        "wood_axe.png": "wooden_axe.png",
        "wood_shovel.png": "wooden_shovel.png",
        "wood_hoe.png": "wooden_hoe.png",
        "gold_sword.png": "golden_sword.png",
        "gold_pickaxe.png": "golden_pickaxe.png",
        "gold_axe.png": "golden_axe.png",
        "gold_shovel.png": "golden_shovel.png",
        "gold_hoe.png": "golden_hoe.png",
        "gold_helmet.png": "golden_helmet.png",
        "gold_chestplate.png": "golden_chestplate.png",
        "gold_leggings.png": "golden_leggings.png",
        "gold_boots.png": "golden_boots.png",
        "gold_ingot.png": "gold_ingot.png",
        "gold_nugget.png": "gold_nugget.png",
        "gold_horse_armor.png": "golden_horse_armor.png",
        "iron_horse_armor.png": "iron_horse_armor.png",
        "diamond_horse_armor.png": "diamond_horse_armor.png",
        "door_wood.png": "oak_door.png",
        "door_iron.png": "iron_door.png",
        "bucket_empty.png": "bucket.png",
        "bucket_water.png": "water_bucket.png",
        "bucket_lava.png": "lava_bucket.png",
        "bucket_milk.png": "milk_bucket.png",
        "fireworks.png": "firework_rocket.png",
        "fireball.png": "fire_charge.png",
        "dye_powder_black.png": "ink_sac.png",
        "dye_powder_red.png": "red_dye.png",
        "dye_powder_green.png": "green_dye.png",
        "dye_powder_brown.png": "cocoa_beans.png",
        "dye_powder_blue.png": "lapis_lazuli.png",
        "dye_powder_purple.png": "purple_dye.png",
        "dye_powder_cyan.png": "cyan_dye.png",
        "dye_powder_silver.png": "light_gray_dye.png",
        "dye_powder_gray.png": "gray_dye.png",
        "dye_powder_pink.png": "pink_dye.png",
        "dye_powder_lime.png": "lime_dye.png",
        "dye_powder_yellow.png": "yellow_dye.png",
        "dye_powder_light_blue.png": "light_blue_dye.png",
        "dye_powder_magenta.png": "magenta_dye.png",
        "dye_powder_orange.png": "orange_dye.png",
        "dye_powder_white.png": "bone_meal.png",
        "book_normal.png": "book.png",
        "book_written.png": "written_book.png",
        "book_writable.png": "writable_book.png",
        "book_enchanted.png": "enchanted_book.png",
        "totem.png": "totem_of_undying.png",
    }

    # Wood prefix renames: planks_oak -> oak_planks, log_oak -> oak_log, leaves_oak -> oak_leaves, sapling_oak -> oak_sapling
    wood_types = ["oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "big_oak"]
    colors = ["white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
              "silver", "cyan", "purple", "blue", "brown", "green", "red", "black"]

    mapping = {}
    matched_count = 0

    for path_old in list_1122:
        parts = path_old.split("/")
        category = parts[3] if len(parts) > 3 else "" # blocks, items, entity, gui, etc.
        fname = parts[-1]

        mapped_path = None

        # 1. Direct path match (e.g. gui/icons.png, entity/creeper/creeper.png)
        if path_old in modern_set:
            mapped_path = path_old

        # 2. Category singularization: blocks -> block, items -> item
        if not mapped_path:
            cat_mod = "block" if category == "blocks" else ("item" if category == "items" else category)
            direct_cat = f"assets/minecraft/textures/{cat_mod}/{fname}"
            if direct_cat in modern_set:
                mapped_path = direct_cat

        # 3. Known name rules
        if not mapped_path and fname in name_rules:
            target_fn = name_rules[fname]
            cat_mod = "block" if category == "blocks" else ("item" if category == "items" else category)
            cand = f"assets/minecraft/textures/{cat_mod}/{target_fn}"
            if cand in modern_set:
                mapped_path = cand
            elif target_fn in modern_by_filename:
                mapped_path = modern_by_filename[target_fn]

        # 4. Wood inversions: planks_X -> X_planks, log_X -> X_log, leaves_X -> X_leaves
        if not mapped_path:
            for prefix in ["planks_", "log_", "leaves_", "sapling_", "door_", "trapdoor_"]:
                if fname.startswith(prefix):
                    rest = fname[len(prefix):-4] # without .png
                    if rest == "big_oak":
                        rest = "dark_oak"
                    prefix_clean = prefix.rstrip("_")
                    if prefix_clean == "planks":
                        cand_fn = f"{rest}_planks.png"
                    elif prefix_clean == "leaves":
                        cand_fn = f"{rest}_leaves.png"
                    elif prefix_clean == "sapling":
                        cand_fn = f"{rest}_sapling.png"
                    elif prefix_clean == "log":
                        cand_fn = f"{rest}_log.png"
                    else:
                        cand_fn = f"{rest}_{prefix_clean}.png"
                    cand = f"assets/minecraft/textures/block/{cand_fn}"
                    if cand in modern_set:
                        mapped_path = cand
                        break

        # 5. Color inversions: hardened_clay_stained_X -> X_terracotta, wool_colored_X -> X_wool, concrete_X -> X_concrete
        if not mapped_path:
            if fname.startswith("wool_colored_"):
                col = fname[len("wool_colored_"):-4]
                if col == "silver": col = "light_gray"
                cand = f"assets/minecraft/textures/block/{col}_wool.png"
                if cand in modern_set: mapped_path = cand
            elif fname.startswith("hardened_clay_stained_"):
                col = fname[len("hardened_clay_stained_"):-4]
                if col == "silver": col = "light_gray"
                cand = f"assets/minecraft/textures/block/{col}_terracotta.png"
                if cand in modern_set: mapped_path = cand
            elif fname.startswith("concrete_powder_"):
                col = fname[len("concrete_powder_"):-4]
                if col == "silver": col = "light_gray"
                cand = f"assets/minecraft/textures/block/{col}_concrete_powder.png"
                if cand in modern_set: mapped_path = cand
            elif fname.startswith("concrete_"):
                col = fname[len("concrete_"):-4]
                if col == "silver": col = "light_gray"
                cand = f"assets/minecraft/textures/block/{col}_concrete.png"
                if cand in modern_set: mapped_path = cand
            elif fname.startswith("glass_stained_"):
                col = fname[len("glass_stained_"):-4]
                if col == "silver": col = "light_gray"
                cand = f"assets/minecraft/textures/block/{col}_stained_glass.png"
                if cand in modern_set: mapped_path = cand

        # 6. Fallback filename lookup
        if not mapped_path and fname in modern_by_filename:
            mapped_path = modern_by_filename[fname]

        if mapped_path:
            mapping[path_old] = mapped_path
            matched_count += 1
        else:
            # Keep original as fallback
            mapping[path_old] = None

    print(f"[Mapping] Matched {matched_count} of {len(list_1122)} textures ({matched_count*100//len(list_1122)}%) to modern counterparts.")
    mapping_file = os.path.join(DATA_DIR, "texture_mapping_1122_to_modern.json")
    with open(mapping_file, 'w', encoding='utf-8') as f:
        json.dump(mapping, f, indent=2)
    print(f"[Mapping] Saved mapping to {mapping_file}")
    return mapping

def generate_modernized_pack(mapping, jar_1122, jar_modern):
    print("[Pack Generator] Creating modernized texture pack for 1.12.2...")
    if os.path.exists(PACK_DIR):
        shutil.rmtree(PACK_DIR)
    os.makedirs(PACK_DIR, exist_ok=True)

    # Open zip files
    with zipfile.ZipFile(jar_1122, 'r') as zf_old, zipfile.ZipFile(jar_modern, 'r') as zf_new:
        modern_namelist = set(zf_new.namelist())
        for old_path, new_path in mapping.items():
            dest_file = os.path.join(PACK_DIR, old_path)
            os.makedirs(os.path.dirname(dest_file), exist_ok=True)

            if new_path and new_path in modern_namelist:
                data = zf_new.read(new_path)
            else:
                data = zf_old.read(old_path)
            
            with open(dest_file, 'wb') as f:
                f.write(data)

    # Add pack.mcmeta for 1.12.2 resource pack (pack_format: 3 for 1.11 - 1.12.2)
    mcmeta = {
        "pack": {
            "pack_format": 3,
            "description": "rMinecraft Modern Texture Pack for 1.12.2 (Backported from Modern Minecraft)"
        }
    }
    with open(os.path.join(PACK_DIR, "pack.mcmeta"), 'w') as f:
        json.dump(mcmeta, f, indent=2)

    # Create ZIP pack
    zip_pack_path = "/home/user/rMinecraft/modern_textures_1.12.2.zip"
    print(f"[Pack Generator] Zipping pack to {zip_pack_path}...")
    with zipfile.ZipFile(zip_pack_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
        for root, dirs, files in os.walk(PACK_DIR):
            for file in files:
                full_path = os.path.join(root, file)
                rel_path = os.path.relpath(full_path, PACK_DIR)
                zipf.write(full_path, rel_path)

    print(f"[Pack Generator] Success! Pack created: {zip_pack_path} ({os.path.getsize(zip_pack_path)} bytes)")

if __name__ == "__main__":
    os.makedirs(DOWNLOAD_DIR, exist_ok=True)
    os.makedirs(DATA_DIR, exist_ok=True)
    
    download_file(JAR_1122_URL, JAR_1122_PATH, "Minecraft 1.12.2")
    download_file(JAR_LATEST_URL, JAR_LATEST_PATH, "Modern Minecraft 26.3")

    list_1122 = extract_texture_list(JAR_1122_PATH, "1.12.2")
    list_modern = extract_texture_list(JAR_LATEST_PATH, "modern_26.3")

    mapping = build_texture_mapping(list_1122, list_modern)
    generate_modernized_pack(mapping, JAR_1122_PATH, JAR_LATEST_PATH)
    print("\n[Complete] All texture lists, mappings, and modernized textures generated successfully!")
