"""The core's tests, every test program at once.

usage: python crates/isostate-core/run_tests.py [--release | --profile NAME] [--quick] [--slowest N]
                                                [--timeout SECONDS] [-- ARGS FOR EVERY TEST PROGRAM]

`cargo test -p isostate-core` builds some forty test programs (the library's own tests, the command's, one
for each file in tests/, and the examples to see that they compile) and runs them one after another, then
the doc tests. Most of its wall time is spent on the three or four longest programs while the machine's
other cores stand idle. This builds the same programs with cargo in the same way and runs them all at the
same time, each in the package's folder as cargo runs it, with the doc tests beside them. It ends with a
non-zero exit code if the build failed, or if any program failed, would not start or ran past its time
limit. Nothing is left out and nothing is added: the same tests run, and must pass, as under cargo test.

The profile is the dev one unless `--release` is given. The dev profile optimises the core as the release
one does (Cargo.toml) and adds debug assertions and overflow checks, so it checks more, not less; the two
golden tests give the same hashes under both. Its build is also the one the viewer's build makes anyway.
`--profile NAME` builds and runs under any profile of the workspace's Cargo.toml (`--profile work`, the
working loop's, is the quickest to build again after a change to the core).

`--quick` leaves out the slowest tests, those named in QUICK_SKIPS below, so that the rest run in about
half a minute, and builds no examples (half the programs to link): for a builder while working
(`--profile work --quick`: a change, its build and these tests in under a minute). The whole set is run
once before a piece is handed over, and the gate runs it all. `--slowest N` lists, at the end, the N tests that took longest and their seconds
(each program's tests are timed by the test harness's own clock, with every program running at once).

It prints one line for each program as it finishes (its seconds, its tests and its verdict), then the
whole output of each program that failed, then a last line: `tests: N programs, P passed, F failed, ...`.
Anything after `--` is handed to every test program (a name to filter by, `--nocapture`, ...).
Environment variables (ISOSTATE_GOLDEN_PRINT and the like) reach the programs as they would under cargo.
Written for the gate after the first step back (19 November 2026), which found the core's tests three
quarters of a landing; docs/rules/README.md, "Running the tests".
"""
import json, os, re, subprocess, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))

# The tests --quick leaves out: the slowest, each by the whole name its program prints (a name that
# two programs share is left out of both).
# Read off `--profile work --slowest 500` on 26 November 2026: each of these took more than ten
# seconds with every program running at once, and together they were 98 per cent of the tests'
# processor time. Most run an island for a century or more, the golden tests among them, so the
# quick set does not hold the port byte for byte: only the whole set does. A slow test added later
# runs in the quick set until its name is put here; a name here that is no longer a test leaves
# nothing out.
QUICK_SKIPS = [
    # ablate
    'an_arm_is_the_sweep_with_the_same_set',

    # accounts
    'a_copy_of_the_island_shares_the_years_put_away',
    'a_game_gone_on_with_has_the_months_it_did_not_watch',
    'a_game_left_in_the_middle_of_a_year_goes_on_with_the_same_books',
    'a_loaded_game_has_its_books_again',
    'a_run_of_one_year_and_towns_not_yet_founded',
    'a_town_and_a_works_can_be_asked_for',
    'a_viewer_that_takes_the_months_as_they_come_is_told_the_years_figures',
    'a_works_founded_at_the_years_end_is_in_that_years_books',
    'the_books_agree_with_the_observer_and_the_families',
    'the_books_are_ready_for_the_island_made_by_water',
    'the_books_are_the_same_books_however_the_game_was_reached',
    'the_books_balance_under_the_game',
    'the_books_balance_under_the_port',
    'the_books_change_nothing',
    'the_books_do_not_change_what_a_save_is_checked_by',
    'the_books_follow_what_a_player_does_between_years',
    'the_books_keep_up_with_the_mines_and_the_canals',
    'the_books_repeat_and_go_into_the_run_file_and_the_live_view',

    # ages
    'a_run_is_the_same_with_the_ages_watched_or_not',
    'off_under_the_port_and_on_in_the_game',
    'the_groups_sum_to_every_towns_people_at_every_years_end',

    # canals
    'a_canal_is_surveyed_then_dug_then_copied',
    'boats_carry_what_goes_far_and_the_railway_takes_it',
    'canals_conserve_money_and_runs_are_deterministic',
    'in_company_with_six_families_canals_are_dug_and_money_is_conserved',
    'navvies_are_taken_on_and_let_go_by_degrees',
    'switched_off_nothing_is_kept_or_recorded',
    'the_links_are_the_network',

    # card
    'each_town_month_count_counts_its_own_case',
    'the_cards_facts_are_what_they_say',

    # care (two runs of island 7 to 1935 side by side; added with it on 26 November 2026)
    'nothing_moves_before_care_and_its_books_balance',

    # carting
    'carters_come_out_of_the_supply_of_hands',
    'carters_taken_on_by_degrees_do_not_set_the_wage_flipping',
    'carting_conserves_money_and_runs_are_deterministic',
    'measuring_the_bill_changes_nothing',

    # chronicle
    'a_decade_keeps_a_handful',
    'a_first_of_its_kind_is_the_first_ever',
    'a_founding_redraws_only_the_districts_it_was_cut_from',
    'a_live_games_marks_end_where_the_recorded_run_puts_them',
    'a_run_from_1755_tells_its_decades_by_the_calendar',
    'a_run_without_families_has_its_history_too',
    'a_works_ordered_by_hand_is_told_as_it_was',
    'every_observer_item_says_why',
    'every_town_has_its_milestones',
    'it_never_touches_the_economy_and_runs_are_deterministic',
    'switched_off_it_leaves_no_trace',
    'the_players_works_no_longer_flood_the_news',
    'towns_and_districts_have_stories',

    # crossings
    'bridges_are_paid_for_and_money_is_conserved_on_the_new_land',

    # edits
    'a_failed_harvest_by_hand_is_that_years_harvest_and_draws_nothing',
    'a_familys_order_with_an_edit_waiting_is_tried_as_before',
    'a_list_with_a_works_by_hand_is_played_before_january_and_refused_after',
    'a_new_edit_refused_leaves_no_trace',
    'a_refused_edit_leaves_no_trace',
    'a_run_with_an_empty_list_of_edits_is_the_run_without',
    'a_session_with_edits_saved_and_loaded_is_the_same_to_the_last_digit',
    'a_town_founded_by_hand_stands_at_the_years_end_with_people_in_it',
    'a_works_built_by_hand_at_a_turn_stands_and_its_cost_is_booked',
    'a_works_by_hand_refused_leaves_no_trace',
    'a_works_closed_by_hand_sends_its_cash_where_close_firm_does',
    'a_works_ordered_by_hand_in_mid_year_waits_for_the_turn',
    'an_idea_found_by_hand_spreads_to_a_second_town',
    'money_by_hand_is_booked_where_the_money_check_reads_it',
    'money_growth_set_to_nothing_by_hand_stops_the_issue',
    'the_new_edits_saved_and_loaded_are_the_same_to_the_last_digit',
    'which_switches_can_be_thrown_in_a_running_island',

    # engel
    'richer_households_buy_less_bread_and_more_comfort',
    'the_curve_conserves_money_and_runs_are_deterministic',

    # families
    'a_passive_family_trains_its_children_in_different_trades',
    'ai_families_play_by_the_same_rules',
    'families_conserve_money_and_runs_are_deterministic',
    'the_first_decision_waits_for_the_player',

    # gauges
    'runs_identical_with_the_gauges_on_and_off',
    'the_gauges_go_into_the_run_file_the_facts_and_the_live_view',
    'the_reckoning_adds_up',
    'without_the_books_the_gauges_do_nothing',

    # golden
    'seed_7_under_the_port_is_unchanged',
    'seed_7_with_six_families_is_unchanged',

    # golden_game: the settled island, and landfall (one town in a corner)
    'seed_7_under_the_games_rules_is_unchanged',
    'landfall_seed_7_under_the_games_rules_is_unchanged',

    # ground
    'after_1900_no_cell_joins_beyond_reach_and_a_belt_of_one_holds',
    'before_1901_the_island_is_the_island_without_it',

    # hands
    'a_town_short_of_hands_shares_out_the_hands_it_has',
    'a_town_with_no_hands_left_hires_nobody',
    'brambank_hires_no_more_than_it_has',
    'rationed_runs_conserve_money_and_are_deterministic',

    # harvest
    'a_bad_year_is_dear_bread_and_a_lower_real_wage_and_the_news_says_so',
    'a_sweep_over_sizes_compares_the_same_weather',
    'harvests_are_recorded_and_the_news_is_sparse',
    'money_is_conserved_and_runs_repeat_with_harvests_on',
    'no_weather_and_the_ports_prices_tell_no_dearth',
    'with_families_money_holds_and_land_is_priced_on_an_average_year',

    # inspect
    'a_towns_ideas_are_every_idea_in_one_of_three_states_and_one_found_by_hand_is_known_there',
    'every_thing_is_told_and_what_is_not_there_is_refused',
    'telling_changes_nothing',
    'the_island_tells_what_the_hand_has_done',

    # island: the island's settings (island 7 to 1780 twice, with a save between; added with them on 27 November 2026)
    'a_game_made_with_three_settings_and_gone_on_with_is_the_one_played_straight_through',
    'each_setting_does_to_the_land_what_it_says',

    # lab
    'an_arm_is_the_run_with_its_edits',
    'the_arms_part_at_the_edit',
    'the_control_is_the_plain_run',

    # land
    'a_lake_is_not_the_sea_and_a_tideway_s_mouth_is_a_harbour',
    'a_run_on_the_new_land_conserves_money_and_is_deterministic',
    'off_by_default_and_the_port_s_island_has_no_landform',
    'one_island_rivers_that_end_where_they_say_and_lakes_behind_their_lips',
    'rivers_reach_the_sea_by_their_estuaries',
    'the_same_seed_makes_the_same_island_and_another_seed_another',
    'the_world_is_whole',
    'water_runs_downhill_to_the_sea',

    # land_century
    'nitrogen_multiplies_the_crop_and_is_paid_for_in_fertiliser',
    'the_land_machines_table_has_its_old_rows_at_stage_one',
    'the_land_waits_for_its_machines',

    # ledger: island 7 to 2040 from each start
    'the_ledger_adds_up_on_the_settled_island',
    'the_ledger_adds_up_at_landfall',

    # longrun
    'a_short_run_is_conserved_with_the_stand_in',
    'the_chain_is_the_observers_and_the_anchor_its_mean_of_five_years',
    'the_facts',
    'the_freight_waits_for_its_year',

    # lorry (its island_7_to_1935 shares tending's name, below)
    'the_hauliers_till_island_7_to_2050',

    # machines
    'machines_stand_for_years',
    'money_is_conserved_and_runs_are_deterministic_with_machines_on',

    # mines
    'a_fund_in_debt_is_repaid_by_its_rents_alone',
    'a_lease_is_whole_and_nobody_sinks_inside_it',
    'a_working_left_idle_is_abandoned_and_one_at_work_is_not',
    'money_is_conserved_and_a_run_is_deterministic',
    'the_viewer_is_given_the_workings',
    'the_workings_give_the_surface_back',
    'under_the_commons_a_fund_in_debt_levies_nobody',
    'what_is_raised_is_counted_against_the_deposit',

    # money
    'the_issue_is_never_below_nothing',
    'the_level_of_money_is_what_the_island_began_with_grown_at_the_rate',
    'the_rate_is_1_764_per_cent_a_year_on_every_island',
    'twice_the_money_at_the_start_is_twice_the_money_in_every_month',
    'what_the_player_does_moves_money_between_purses_and_makes_none',

    # new_good
    'a_good_of_1950_leaves_island_7_as_it_was_to_1949',
    'idle_goods_leave_island_2_as_it_was_to_2050_under_the_game',
    'idle_goods_leave_island_7_as_it_was_under_the_port',

    # owners
    'island_7_to_1850_holds_its_works_by_shares_that_add_up',
    'off_is_the_game_without_it',

    # people
    'a_sailing_is_news_of_its_own_kind',
    'a_town_with_a_tiny_market_area_does_not_run_away',
    'in_company_no_wage_runs_away_or_flips',
    'people_conserve_money_and_runs_are_deterministic',

    # power
    'at_stage_one_the_island_is_the_island_as_it_was',
    'electricity_does_not_keep_and_a_town_without_a_station_has_none',

    # prices
    'forty_years_conserve_money_and_come_out_the_same_twice',
    'the_markup_steadies_iron_and_the_farm_share',
    'the_wage_shortcut_agrees_with_the_plain_bisection',

    # railways
    'a_line_opens_only_onto_track_that_is_there',
    'branches_carry_the_network_past_the_ports',
    'railways_conserve_money_and_runs_are_deterministic',
    'the_news_keeps_the_trunks_and_folds_the_branches',

    # roads
    'paid_roads_conserve_money_and_pay_their_way',
    'switched_off_nothing_is_kept',

    # rulebook
    'the_dial_is_offered_in_multiples_and_set_by_hand_to_tenfold',
    'what_the_walk_says_of_an_offer_is_what_the_edit_does_with_it',

    # save
    'a_decision_answered_late_in_the_year_takes_the_place_of_its_default',
    'a_file_that_says_what_no_game_could_have_written_is_refused',
    'a_game_to_2050_saved_after_1900_loads_again_to_the_same_island',
    'a_game_with_as_many_families_as_there_are_names_starts_and_plays',
    'a_games_last_year_is_kept_in_its_save_and_checked',
    'a_month_at_a_time_is_the_same_game_as_a_year_at_a_time',
    'a_save_made_at_a_turn_before_its_orders_are_carried_out_keeps_them',
    'a_save_made_before_the_switch_is_played_again_on_the_old_island',
    'a_save_the_rules_no_longer_agree_with_says_so',
    'every_save_says_its_land_and_is_played_again_on_it',
    'orders_given_in_the_middle_of_a_year_wait_for_its_end',
    'orders_that_name_anything_at_all_are_refused_or_carried_out_and_the_game_goes_on',
    'thirty_years_saved_and_loaded_are_the_same_to_the_last_digit',

    # shelves
    'a_debt_stays_with_the_merchants_who_owe_it',
    'a_district_that_goes_and_comes_back',
    'a_town_that_gives_away_its_last_district',
    'a_town_that_takes_three_districts_in_one_year',
    'no_hungry_january_after_a_town_takes_a_district',
    'the_shelves_go_with_a_district_to_an_old_town_and_nothing_is_lost',

    # squares
    'the_book_holds_every_year_on_islands_1_to_8',
    'a_works_keeps_its_first_square_while_it_stands',
    'nothing_moves_with_the_switch',

    # state
    'a_chip_overrides_the_book_from_the_year_it_is_set',
    'the_book_pays_from_1901_and_the_treasury_balances',
    'the_state_with_nothing_in_it_is_the_run_without_it',

    # sweep
    'a_full_works_table_is_said_plainly',
    'a_tiny_sweep_writes_finite_facts',
    'no_fact_main_had_is_lost_or_renamed',
    'the_facts_no_file_holds_agree_with_the_record',

    # tasks
    'computing_off_is_off_and_on_stays_in_reach',
    'island_7_to_1925',
    'island_7_to_1925_on_the_land',
    'machines_work_once_they_stand',
    'models_off_is_off_and_held_from_the_first_month',
    'no_station_no_drive',
    'office_machines_hold_no_more_than_a_quarter',
    'starved_of_plant_nothing_is_held',
    'the_ceiling_waits_for_1950',

    # tending
    'island_7_to_1935',
    'own_run_feeds_no_shortage_of_its_own_good',

    # towns
    'a_long_deep_decline_gives_ground_up_one_cell_a_year',
    'a_new_town_is_reckoned_at_its_donors_wage',
    'a_town_keeps_its_ground_and_grows_from_it',
    'kept_ground_conserves_money_and_is_deterministic',
    'off_is_the_port_and_on_is_the_game',
    'the_record_and_the_live_view_carry_the_three_layers',

    # traffic
    'the_traffic_record_changes_nothing_played_and_is_the_roads_own_walk',

    # transition
    'island_7_keeps_a_sound_measure_in_every_town',
    'off_is_the_game_without_it',

    # tree
    'money_is_conserved_and_runs_are_deterministic_with_the_tree_laid',
    'plant_waits_and_the_bricklayers_are_served',
    'the_island_opens_with_builders_and_they_take_the_quarries_work',
    'the_tree_alone_runs_under_the_ports_rules',
    'without_a_stage_plant_waiting_does_nothing',

    # university
    'a_university_is_founded_and_closed_by_hand',
    'the_book_founds_a_university_in_the_seat_and_its_students_come',
    'with_the_university_off_the_purse_has_its_seat_and_nothing_more',

    # who_works
    'nobody_is_counted_twice',
    'off_is_off_and_to_1900_nothing_moves',

    # woods
    'it_is_found_after_its_year_spreads_and_grows_the_woods_back_faster',
    'with_the_rule_off_it_is_never_sought_and_the_island_is_the_island_without_it',

    # works
    'a_family_extends_the_works_it_has',
    'a_works_pays_for_its_order_before_its_owners',
    'the_life_cycle_conserves_money_and_is_deterministic',
    'the_works_of_1749_stand_on_cells_of_their_own',
    'works_grow_where_they_stand',
]


def main(argv):
    release, profile_name, quick, slowest, timeout, passed_on = False, None, False, 0, 1800.0, []
    if "--" in argv:
        k = argv.index("--")
        argv, passed_on = argv[:k], argv[k + 1:]
    it = iter(argv)
    try:
        for a in it:
            if a == "--release":
                release = True
            elif a == "--profile":
                profile_name = next(it)
            elif a == "--quick":
                quick = True
            elif a == "--slowest":
                slowest = int(next(it))
            elif a == "--timeout":
                timeout = float(next(it))
            elif a in ("-h", "--help"):
                print(__doc__)
                return 0
            else:
                print("unknown option %r\n%s" % (a, __doc__.splitlines()[2]), file=sys.stderr)
                return 2
    except (StopIteration, ValueError):
        print("option %r wants a value after it\n%s" % (a, __doc__.splitlines()[2]), file=sys.stderr)
        return 2
    if release and profile_name:
        print("--release and --profile cannot both be given (--release is --profile release)", file=sys.stderr)
        return 2
    if quick:
        # --exact: a name leaves out that test alone, not every test whose name contains it (it also
        # makes a name given after -- match whole names only)
        passed_on = passed_on + ["--exact"] + [x for name in QUICK_SKIPS for x in ("--skip", name)]
    # the test harness's own clock for each test: an unstable option of the harness, which the
    # variable below opens to it in the test programs only (the build and the doc tests never see it)
    timed = ["-Zunstable-options", "--report-time"] if slowest else []
    profile = ["--release"] if release else (["--profile", profile_name] if profile_name else [])
    t0 = time.time()
    # 1. the build, as cargo test does it (the examples are built too, so that one that no longer
    # compiles fails here as it fails there); cargo's own messages go to the terminal
    build = ["cargo", "test", "-p", "isostate-core", "--no-run", "--message-format=json-render-diagnostics"] + profile
    if quick:
        # the library, the command and tests/, without the examples (--tests: every target that is tested)
        build.append("--tests")
    proc = subprocess.run(build, cwd=HERE, stdout=subprocess.PIPE, text=True, errors="replace")
    programs = []
    for line in proc.stdout.splitlines():
        try:
            m = json.loads(line)
        except ValueError:
            continue
        if m.get("reason") == "compiler-artifact" and m.get("profile", {}).get("test") and m.get("executable"):
            kind = m["target"]["kind"][0]
            name = m["target"]["name"] if kind == "test" else "%s (%s)" % (m["target"]["name"], kind)
            programs.append((name, [m["executable"]] + passed_on + timed))
    if proc.returncode != 0:
        print("tests: the build failed (%s), so nothing was run" % " ".join(build), file=sys.stderr)
        return proc.returncode or 1
    if not programs:
        print("tests: the build gave no test programs", file=sys.stderr)
        return 1
    built = time.time() - t0
    # the doc tests, which cargo test runs last, as one more program beside the others
    programs.append(("doc tests", ["cargo", "test", "-p", "isostate-core", "--doc"] + profile + (["--"] + passed_on if passed_on else [])))
    print("built %d test programs in %.0f s; running them and the doc tests all at once" % (len(programs) - 1, built), flush=True)
    # 2. every program at once, each in the package's folder, as cargo runs it
    env = dict(os.environ, CARGO_MANIFEST_DIR=HERE, CARGO_PKG_NAME="isostate-core")
    env_timed = dict(env, RUSTC_BOOTSTRAP="1") if slowest else env
    logs = tempfile.mkdtemp(prefix="isostate-tests-")
    running = []
    for name, cmd in programs:
        log = open(os.path.join(logs, re.sub(r"[^A-Za-z0-9_.-]", "_", name) + ".log"), "w+", encoding="utf-8", errors="replace")
        try:
            p = subprocess.Popen(cmd, cwd=HERE, env=env if name == "doc tests" else env_timed, stdout=log, stderr=subprocess.STDOUT)
        except OSError as e:
            log.write("could not start %s: %s\n" % (cmd[0], e))
            p = None
        running.append((name, p, time.time(), log))
    done, times = [], []
    while running:
        for r in list(running):
            name, p, start, log = r
            rc = -1 if p is None else p.poll()
            if rc is None and time.time() - start > timeout:
                p.kill()
                p.wait()
                log.write("\n(stopped: still running after %.0f s, the time limit)\n" % timeout)
                rc = -2
            if rc is None:
                continue
            seconds = time.time() - start
            log.flush()
            log.seek(0)
            text = log.read()
            log.close()
            counts = [0, 0, 0, 0]
            for m in re.finditer(r"^test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored; \d+ measured; (\d+) filtered out", text, flags=re.M):
                for k in range(4):
                    counts[k] += int(m.group(k + 1))
            for m in re.finditer(r"^test (\S+) \.\.\. \w+ <([\d.]+)s>", text, flags=re.M):
                times.append((float(m.group(2)), name, m.group(1)))
            ok = rc == 0
            print("  %-6s %6.1f s  %-24s %d passed, %d failed, %d ignored%s" % ("ok" if ok else "FAILED", seconds, name, *counts[:3],
                ", %d left out" % counts[3] if counts[3] else ""), flush=True)
            done.append((name, ok, rc, counts, text))
            running.remove(r)
        time.sleep(0.2)
    failed = [d for d in done if not d[1]]
    for name, _, rc, _, text in failed:
        why = {-1: "it would not start", -2: "it ran past the time limit"}.get(rc, "exit code %d" % rc)
        print("\n---- %s failed (%s); its output:\n%s" % (name, why, text.rstrip()))
    total = [sum(d[3][k] for d in done) for k in range(4)]
    end = time.time()
    if slowest:
        print("\nthe %d slowest tests:" % min(slowest, len(times)))
        for seconds, program, test in sorted(times, reverse=True)[:slowest]:
            print("  %7.1f s  %-16s %s" % (seconds, program, test))
    print("\ntests: %d programs, %d passed, %d failed, %d ignored%s; %d program(s) failed; built in %.0f s, ran in %.0f s, %.0f s in all%s" % (
        len(done), total[0], total[1], total[2], ", %d left out by --quick" % total[3] if quick else "", len(failed), built,
        end - t0 - built, end - t0, "; their output is kept in " + logs if failed else ""), flush=True)
    if not failed:
        # the output is worth keeping only when something failed, and then it is printed above
        for f in os.listdir(logs):
            os.remove(os.path.join(logs, f))
        os.rmdir(logs)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
