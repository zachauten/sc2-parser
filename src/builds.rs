use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Debug;

pub struct Builds {
  tokens: HashMap<String, u32>,
  cached_token_probability: HashMap<String, f32>,
  pub probability: HashMap<String, f32>,
  build_token_paths: Vec<(String, f32, u8)>,
  pub token_paths: Vec<(String, f32, u8)>,
  pub skipped_builds: Vec<String>,
  pub build_comparison_information: HashMap<String, f32>,
  // cluster_comparisons: HashMap<>,
}

const MAX_TOKEN_SIZE: usize = 4;
const SECTION_SEPARATOR: &str = "__";
const TOKEN_SEPARATOR: char = ':';
const TOKEN_TERMINATOR: &str = "NONE";
const BUILDING_SEPARATOR: &str = ",";
const BUILDING_SEPARATOR_CHAR: char = ',';
const BUILD_SEPARATOR: &str = "--";

impl Builds {
  pub fn new() -> Builds {
    Builds {
      tokens: HashMap::new(),
      cached_token_probability: HashMap::new(),
      probability: HashMap::new(),
      build_token_paths: vec![],
      token_paths: vec![],
      skipped_builds: vec![],
      build_comparison_information: HashMap::new(),
    }
  }

  pub fn generate_tokens(&mut self, build: &Vec<String>, token_prefix: String) {
    for i in 0..build.len() {
      for window_size in 1..MAX_TOKEN_SIZE + 1 {
        let tokens = &build[i..i + window_size];
        let mut current_token = tokens[0].clone();
        let mut next_token = TOKEN_TERMINATOR;

        if tokens.len() > 1 && tokens.len() != build.len() {
          current_token = tokens[..tokens.len() - 1].join(BUILDING_SEPARATOR);
          next_token = &tokens[tokens.len() - 1];
        }

        let identifier_token = format!("{token_prefix}{SECTION_SEPARATOR}{current_token}{SECTION_SEPARATOR}{next_token}");
        self.tokens.entry(identifier_token).and_modify(|count| *count += 1).or_insert(1);

        if i + window_size >= build.len() {
          break;
        }
      }
    }
  }

  pub fn generate_token_distributions(&mut self) {
    let mut token_totals: HashMap<String, u32> = HashMap::new();
    for (key, token_count) in &self.tokens {
      let values: Vec<&str> = key.split(SECTION_SEPARATOR).collect();
      let prefix = values[0];
      let current_token = values[1];
      let next_token = values[2];

      let current_token_identifier = if next_token == TOKEN_TERMINATOR {
        format!("{prefix}{SECTION_SEPARATOR}{TOKEN_TERMINATOR}")
      } else {
        format!("{prefix}{SECTION_SEPARATOR}{current_token}")
      };

      token_totals
        .entry(current_token_identifier)
        .and_modify(|count| *count += token_count)
        .or_insert(*token_count);
    }

    for (key, count) in &self.tokens {
      let values: Vec<&str> = key.split(SECTION_SEPARATOR).collect();
      let prefix = values[0];
      let current_token = values[1];
      let next_token = values[2];

      let current_token_identifier = if next_token == TOKEN_TERMINATOR {
        format!("{prefix}{SECTION_SEPARATOR}{TOKEN_TERMINATOR}")
      } else {
        format!("{prefix}{SECTION_SEPARATOR}{current_token}")
      };

      let current_token_total = match token_totals.get(&current_token_identifier) {
        Some(total) => total,
        None => panic!("Couldn't find total for current token: {:?}", current_token),
      };

      // not enough occurrences to generate a meaningful probability
      if *current_token_total < 10 {
        continue;
      }

      let probability = *count as f32 / *current_token_total as f32;
      //  println!("token probability\n{:?}\n{:?}\n{:?} {:?} {:?}\n", key, current_token_identifier, count, current_token_total, probability);
      self.probability.insert(key.to_string(), probability);
    }
  }

  pub fn generate_token_paths(&mut self, build: &Vec<String>, build_prefix: String) {
    self.generate_next_path(
      String::new(),
      0,
      1.0,
      build,
      build_prefix.as_str(),
      0,
    );

    // sort by token path probabilities
    self.build_token_paths
      .sort_by(|a, b|
        a.1
          .partial_cmp(&b.1)
          .expect("path probabilities should be floats"));

    // push highest probability path to global token paths
    if let Some(calculated_path) = self.build_token_paths.last() {
      self.token_paths.push(calculated_path.to_owned());
    } else {
      self.skipped_builds.push(build.join(BUILDING_SEPARATOR));
    }

    // clear build token paths for next build
    self.build_token_paths.clear();
  }

  fn generate_next_path<'a>(
    &mut self,
    current_path: String,
    current_path_length: usize,
    path_probability: f32,
    build: &'a Vec<String>,
    build_prefix: &str,
    build_index: usize,
  ) {
    for token_window in 1..MAX_TOKEN_SIZE + 1 {
      // base case for recursion
      if build_index + token_window > build.len() {
        // ensure we have a path the same size as our original build
        // paths may be shorter if the last token was skipped due to low probability
        if current_path_length == build.len() {
          let token_path = format!("{build_prefix}{SECTION_SEPARATOR}{current_path}");
          self.build_token_paths.push((token_path, path_probability, current_path_length as u8));
        }
        break;
      }

      let tokens = &build[build_index..build_index + token_window];

      // assume only 1 token is present
      let mut current_token = tokens[0].clone();
      let mut next_token = TOKEN_TERMINATOR;

      // if more than 1 token, update values
      // check against build length since token could encompass entire build if build < max token size
      if tokens.len() > 1 && tokens.len() != build.len() {
        current_token = tokens[..tokens.len() - 1].join(BUILDING_SEPARATOR);
        next_token = &tokens[tokens.len() - 1];
      }

      let mut next_path_probability = path_probability.clone();
      let identifier_token = format!("{build_prefix}{SECTION_SEPARATOR}{current_token}{SECTION_SEPARATOR}{next_token}");

      // if we don't have a conditional probability for the tokens there was < 10 occurrences
      if !self.probability.contains_key(&identifier_token) {
        continue;
      }

      // check to see if we've previously computed the probability of this building sequence
      if let Some(token_sequence_probability) = self.cached_token_probability.get(&identifier_token) {
        next_path_probability *= token_sequence_probability;
      } else {
        let mut token_sequence_probability = path_probability.clone();
        let mut token_fragment_window = tokens.len();

        // generate fragments of the current token backwards from the full token
        // if we find a token whose sequence we've already computed we can use that value and exit early
        for i in 0..tokens.len() {
          token_fragment_window  -= i;

          // assume unigram. e.g. only 1 token is present
          let mut current_token_fragment = tokens[0].clone();
          let mut next_token_fragment = TOKEN_TERMINATOR;

          // if more than unigram, update values
          if token_fragment_window > 1 && token_fragment_window != build.len() {
            current_token_fragment = tokens[..token_fragment_window - 1].join(BUILDING_SEPARATOR);
            next_token_fragment = &tokens[token_fragment_window - 1];
          }

          let identifier_token_fragment = format!("{build_prefix}{SECTION_SEPARATOR}{current_token_fragment}{SECTION_SEPARATOR}{next_token_fragment}");

          // if we find a subsequence that has already been computed, use the cached value and finish the computation
          if let Some(token_fragment_sequence_probability) = self.cached_token_probability.get(&identifier_token_fragment) {
            token_sequence_probability *= token_fragment_sequence_probability;
            break;
          }

          // get the probability for the current fragment and add it to the sequence probability
          // this should always exist because otherwise we would have already bailed from generating the path
          match self.probability.get(&identifier_token_fragment) {
            Some(token_fragment_probability) => token_sequence_probability *= token_fragment_probability,
            None => panic!("Couldn't find fragment probability on iteration {:?} {:?} {:?}", i, current_token, identifier_token_fragment),
          }
        }

        // add the current building sequence probability to the cache
        self.cached_token_probability.insert(identifier_token, token_sequence_probability);
        next_path_probability *= token_sequence_probability;
      }

      let mut next_path = current_path.clone();
      if next_path != "" {
        next_path.push(TOKEN_SEPARATOR);
      }
      next_path.push_str(tokens.join(BUILDING_SEPARATOR).as_str());
      let next_path_length = current_path_length + tokens.len();

      self.generate_next_path(
        next_path,
        next_path_length,
        next_path_probability,
        build,
        build_prefix,
        build_index + token_window,
      );
    }
  }

  // https://github.com/python/cpython/blob/c6b84a727c9299f24edbab4105ce47e9f2bae199/Lib/difflib.py#L305
  fn find_longest_match<S: Into<String> + Debug>(
    build: &Vec<S>,
    other_build: &Vec<S>,
    build_low: u8,
    build_high: u8,
    other_build_low: u8,
    other_build_high: u8,
  ) -> (u8, u8, u8) where S: Eq + Hash {
    let mut other_build_mapping: HashMap<&S, Vec<u8>> = HashMap::new();
    for (index, building) in other_build.iter().enumerate() {
      other_build_mapping
        .entry(building)
        .and_modify(|indexes| indexes.push(index as u8))
        .or_insert(vec![index as u8]);
    }

    let (
      mut best_match_lower_bound,
      mut best_match_upper_bound,
      mut best_match_size,
    ) = (build_low, other_build_low, 0);

    let mut match_sizes: HashMap<i8, i8> = HashMap::new();
    for building_index in build_low..build_high {
      let mut new_match_sizes: HashMap<i8, i8> = HashMap::new();
      if let Some(building_match_indexes) = other_build_mapping.get(&build[building_index as usize]) {
        for other_build_index in building_match_indexes {
          if *other_build_index < other_build_low {
            continue;
          }

          if *other_build_index >= other_build_high {
            break;
          }

          let size_lookup_index: i8 = *other_build_index as i8 - 1;
          let new_match_size: i8 = match match_sizes.get(&size_lookup_index) {
            Some(match_length) => *match_length + 1,
            None => 1,
          };
          new_match_sizes.insert(*other_build_index as i8,  new_match_size);

          if new_match_size > best_match_size {
            (
              best_match_lower_bound,
              best_match_upper_bound,
              best_match_size,
            ) = (
              (building_index + 1) - new_match_size as u8,
              (other_build_index + 1) - new_match_size as u8,
              new_match_size,
            );
          }
        }
      }
      match_sizes = new_match_sizes;
    }

    while
      (best_match_lower_bound > build_low) &&
      (best_match_upper_bound > other_build_low) &&
      (build[(best_match_lower_bound - 1) as usize] == other_build[(best_match_upper_bound - 1) as usize])
    {
      (
        best_match_lower_bound,
        best_match_upper_bound,
        best_match_size,
      ) = (
        best_match_lower_bound - 1,
        best_match_upper_bound - 1,
        best_match_size + 1,
      );
    }

    while
      ((best_match_lower_bound + best_match_size as u8) < build_high) &&
      ((best_match_upper_bound + best_match_size as u8) > other_build_high) &&
      (build[(best_match_lower_bound + best_match_size as u8) as usize] == other_build[(best_match_upper_bound + best_match_size as u8) as usize])
    {
      best_match_size += 1;
    }

    (best_match_lower_bound, best_match_upper_bound, best_match_size as u8)
  }

  // https://github.com/python/cpython/blob/c6b84a727c9299f24edbab4105ce47e9f2bae199/Lib/difflib.py#L421
  fn get_matching_blocks<S: Into<String> + Eq + Hash + Debug>(
    build: &Vec<S>,
    other_build: &Vec<S>,
  ) -> Vec<(u8, u8, u8)> {
    let mut queue: Vec<(u8, u8, u8, u8)> = vec![(0, build.len() as u8, 0, other_build.len() as u8)];
    let mut matching_blocks = vec![];

    while queue.len() != 0 {
      let (
        build_low_index,
        build_high_index,
        other_build_low_index,
        other_build_high_index,
      ) = queue.pop().unwrap();

      let longest_match = Builds::find_longest_match(
        build,
        other_build,
        build_low_index,
        build_high_index,
        other_build_low_index,
        other_build_high_index,
      );
      // println!("longest match {:?}", longest_match);
      let (build_match_index, other_build_match_index, match_length) = longest_match;

      if match_length != 0 {
        matching_blocks.push(longest_match);
        if
          build_low_index < build_match_index &&
          other_build_low_index < other_build_match_index
        {
          queue.push((
            build_low_index,
            build_match_index,
            other_build_low_index,
            other_build_match_index,
          ));
        }

        if
          (build_match_index + match_length) < build_high_index &&
          (other_build_match_index + match_length) < other_build_high_index
        {
          queue.push((
            build_match_index + match_length,
            build_high_index,
            other_build_match_index + match_length,
            other_build_high_index,
          ));
        }
      }
    }
    matching_blocks.sort();

    let (
      mut previous_build_match_index,
      mut previous_other_build_match_index,
      mut previous_match_length,
    ) = (0, 0, 0);
    let mut non_adjacent = vec![];
    for (build_match_index, other_build_match_index, match_length) in &matching_blocks {
      if
        (previous_build_match_index + previous_match_length) == *build_match_index &&
        (previous_other_build_match_index + previous_match_length) == *other_build_match_index
      {
        previous_match_length += match_length;
      } else {
        if previous_match_length != 0 {
          non_adjacent.push((
            previous_build_match_index,
            previous_other_build_match_index,
            previous_match_length,
          ));
        }
        (
          previous_build_match_index,
          previous_other_build_match_index,
          previous_match_length,
        ) = (
          *build_match_index,
          *other_build_match_index,
          *match_length,
        )
      }
    }

    if previous_match_length != 0 {
      non_adjacent.push((
        previous_build_match_index,
        previous_other_build_match_index,
        previous_match_length,
      ));
    }

    non_adjacent
  }

  pub fn compare_builds(&mut self) {
    let mut build_path_mappings: HashMap<&String, Vec<&str>> = HashMap::new();
    // let mut build_building_count: HashMap<String, u8> = HashMap::new();
    for (path, _, _) in &self.token_paths {
      let mapped_path: Vec<&str> = path
        .split(SECTION_SEPARATOR)
          .collect::<Vec<&str>>()[1]
        .split(
          |c| c == TOKEN_SEPARATOR || c == BUILDING_SEPARATOR_CHAR
        ).collect();

      // for building in &mapped_path {
      //   let building_identifier = format!("{}__{building}", mapped_path.join(","));
      //   build_building_count
      //     .entry(building_identifier)
      //     .and_modify(|count| *count += 1)
      //     .or_insert(1);
      // }

      build_path_mappings.insert(path, mapped_path);
    }

    let mut build_missing_buildings = vec![];
    let mut other_build_missing_buildings = vec![];
    let mut build_missing_building_count: HashMap<&str, u8> = HashMap::new();
    let mut other_build_missing_building_count: HashMap<&str, u8> = HashMap::new();
    for (path, build) in &build_path_mappings {
      for (other_path, other_build) in &build_path_mappings {
        // skip when we encounter the same game
        if path == other_path {
          continue;
        }

        let build_prefix = path.split(SECTION_SEPARATOR).collect::<Vec<&str>>()[0];
        let other_build_prefix = other_path.split(SECTION_SEPARATOR).collect::<Vec<&str>>()[0];

        // only generate comparisons for builds from the same race and same matchup
        if build_prefix != other_build_prefix {
          continue;
        }

        let joined_build = build.join(BUILDING_SEPARATOR);
        let joined_other_build = other_build.join(BUILDING_SEPARATOR);

        // if the builds being compared are the same, there is no difference in information
        if joined_build == joined_other_build {
          continue;
        }

        let mut comparison_builds = vec![
          format!("{build_prefix}{SECTION_SEPARATOR}{joined_build}"),
          format!("{other_build_prefix}{SECTION_SEPARATOR}{joined_other_build}"),
        ];
        comparison_builds.sort();
        let build_comparison_identifier = comparison_builds.join(BUILD_SEPARATOR);

        // println!("comparing builds\n{:?}\n{:?}", joined_build, joined_other_build);

        // we already have information on this build comparison
        if let Some(_) = self.build_comparison_information.get(&build_comparison_identifier) {
          continue;
        }

        for matching_block in Builds::get_matching_blocks(build, other_build) {
          // rounds to 2 dp
          let build_matching_range = matching_block.0..matching_block.0 + matching_block.2;
          for i in 0..build.len() as u8 {
            // only push buildings which are not in the matching block ranges
            if !build_matching_range.contains(&i) {
              build_missing_buildings.push(build[i as usize]);
            }
          }

          // -----

          let other_build_matching_range = matching_block.1..matching_block.1 + matching_block.2;
          for i in 0..other_build.len() as u8 {
            // only push buildings which are not in the matching block ranges
            if !other_build_matching_range.contains(&i) {
              other_build_missing_buildings.push(other_build[i as usize]);
            }
          }
        }

        build_missing_buildings.sort();
        other_build_missing_buildings.sort();

        // if one build is a subset of the other there may be no matches for the subset build
        if build_missing_buildings.len() != 0 {
          let mut prev_building = build_missing_buildings[0];
          let mut missing_building_count = 0;
          for building in &build_missing_buildings {
            if *building == prev_building {
              missing_building_count += 1;
            } else {
              build_missing_building_count.insert(prev_building, missing_building_count);
              prev_building = building;
              missing_building_count = 1;
            }
          }
          // captures the last building
          build_missing_building_count.insert(prev_building, missing_building_count);
        }

        // if one build is a subset of the other there may be no matches for the subset build
        if other_build_missing_buildings.len() != 0 {
          let mut other_prev_building = other_build_missing_buildings[0];
          let mut other_missing_building_count = 0;
          for building in &other_build_missing_buildings {
            if *building == other_prev_building {
              other_missing_building_count += 1;
            } else {
              other_build_missing_building_count.insert(other_prev_building, other_missing_building_count);
              other_prev_building = building;
              other_missing_building_count = 1;
            }
          }
          // captures the last building
          other_build_missing_building_count.insert(other_prev_building, other_missing_building_count);
        }

        // ---

        let mut match_information_difference = 0.0;

        // calculate and sum td-idf values for each building in both builds being compared which is not a match
        // this calculates the total information difference between builds
        for (index, building) in build.iter().enumerate() {
          // building is matching with other build, not missing
          if let None = build_missing_building_count.get(building) {
            continue;
          }

          // building information generated from independent probability of the building
          let token_identifier = format!("{build_prefix}{SECTION_SEPARATOR}{building}{SECTION_SEPARATOR}{TOKEN_TERMINATOR}");
          let building_information = -self.probability[&token_identifier].log2();

          let mut tf_idf = 0.0;
          if index < 10 {
            // https://en.wikipedia.org/wiki/Tf%E2%80%93idf#Term_frequency%E2%80%93inverse_document_frequency
            // 1 / frequency so tf-idf value can be scaled based on building index
            tf_idf = (1.0 / (build_missing_building_count[building] as f32)) * building_information;
            if index > 5 {
              // scaling tf-idf based on building index to prevent later buildings having outsized
              // effect on comparison information since later buildings naturally have less relevance
              tf_idf *= 1.0 - ((index - 5) as f32 / 5.0);
            }
            // println!("tf-idf inside loop {:?} {:?} {:?}", tf_idf, build_missing_building_count[building], building_information);
          }
          match_information_difference += tf_idf;
          // println!("build tf-idf {:?} {:?} {:?} {:?}", match_information_difference, tf_idf, building_information, build_missing_building_count[building]);
        }

        for (index, other_building) in other_build.iter().enumerate() {
          // building is matching with other build, not missing
          if let None = other_build_missing_building_count.get(other_building) {
            continue;
          }

          // building information generated from independent probability of the building
          let token_identifier = format!("{other_build_prefix}{SECTION_SEPARATOR}{other_building}{SECTION_SEPARATOR}{TOKEN_TERMINATOR}");
          let building_information = -self.probability[&token_identifier].log2();

          let mut tf_idf = 0.0;
          if index < 10 {
            // https://en.wikipedia.org/wiki/Tf%E2%80%93idf#Term_frequency%E2%80%93inverse_document_frequency
            // 1 / frequency so tf-idf value can be scaled based on building index
            tf_idf = (1.0 / (other_build_missing_building_count[other_building] as f32)) * building_information;
            if index > 5 {
              // scaling tf-idf based on building index to prevent later buildings having outsized
              // effect on comparison information since later buildings naturally have less relevance
              tf_idf *= 1.0 - ((index - 5) as f32 / 5.0);
            }
            // println!("tf-idf inside loop {:?} {:?} {:?}", tf_idf, other_build_missing_building_count[other_building], building_information);
          }
          match_information_difference += tf_idf;
          // println!("other build tf-idf {:?} {:?} {:?} {:?}", match_information_difference, tf_idf, other_build_missing_building_count[other_building], building_information);
        }

        // reset matching buildings for next build comparison
        build_missing_buildings.clear();
        other_build_missing_buildings.clear();

        // reset missing builing count
        build_missing_building_count.clear();
        other_build_missing_building_count.clear();

        // if match_information_difference < 10.0 {
        let rounded_information_difference = (match_information_difference * 100.0).round() / 100.0;
        self.build_comparison_information.insert(build_comparison_identifier, rounded_information_difference);
      }
    }
  }

  // pub fn generate_clusters(&mut self) {
  //   while true {

  //   }
  //   cluster_comparisons = {}
  //   for (build_id, other_id), diff in build_comparisons.items():
  //       if build_id in build_clusters and other_id in build_clusters:
  //           cluster_comparisons[(build_id, other_id)] = diff

  //   if not cluster_comparisons:
  //       break

  //   sorted_comparisons = sorted(
  //       cluster_comparisons.items(),
  //       key=lambda build: build[1],
  //   )

  //   completed = False
  //   for min_comparison_builds, min_comparison_diff in sorted_comparisons:
  //       if min_comparison_diff > MAX_COMPARISON_DIFF:
  //           break

  //       # cross check cluster builds
  //       cluster_complete_linkage = True
  //       for build_id in min_comparison_builds:
  //           other_comparison_id = min_comparison_builds[0] if min_comparison_builds[1] == build_id else min_comparison_builds[1]
  //           for other_id in build_clusters[build_id]:
  //               cross_cluster_diff = build_comparisons[tuple(sorted([other_comparison_id, other_id]))]
  //               if cross_cluster_diff > MAX_COMPARISON_DIFF:
  //                   # print(other_comparison_id, other_id, cross_cluster_diff)
  //                   cluster_complete_linkage = False
  //                   break

  //           if not cluster_complete_linkage:
  //               break

  //       if not cluster_complete_linkage:
  //           continue

  //       max_build_count = -1
  //       max_build_id = None
  //       for build_id in min_comparison_builds:
  //           if build_list[build_id][1] > max_build_count:
  //               max_build_count = build_list[build_id][1]
  //               max_build_id = build_id

  //       other_build_id = min_comparison_builds[0] if min_comparison_builds[1] == max_build_id else min_comparison_builds[1]
  //       build_clusters[max_build_id].extend(build_clusters[other_build_id])
  //       build_clusters[max_build_id].append(other_build_id)
  //       del build_clusters[other_build_id]
  //       completed = True
  //       break

  //   if not completed:
  //       break

  // }
}