#!/usr/bin/env node
// Usage: node gen_bonds.js [count] > bonds.json
// Requires: npm install @faker-js/faker

import { faker } from "@faker-js/faker";

const COUNT = parseInt(process.argv[2] ?? "4", 10);

const STATES = ["DRAFT", "ACTIVE", "REDEEMABLE", "EXPIRED", "SETTLED", "RECLAIMED"];
const CURRENCIES = ["xmr", "zec", "btc", "eth"];
const LOCK_MECHANISMS = {xmr:"dleq_xmr",zec:"dleq_zec_sapling",btc:"ptlc_btc",eth:"eth_escrow"};
const KEY_ALGORITHMS  = {xmr:"ed25519",zec:"jubjub",btc:"secp256k1",eth:"secp256k1"};
const ETH_CHAIN_IDS   = [1, 11155111, 17000];
const GOAL_TYPES = ["quantitative", "qualitative", "hybrid"];
const OPERATORS = ["eq", "gte", "lte", "gt", "lt"];
const ORACLE_ROLES = ["quantitative", "qualitative", "both"];

const DATA_ID_PREFIXES = [
  "usgov.senate",
  "usgov.fda",
  "usgov.cbo",
  "clinicaltrials.gov",
  "valar.atomics",
  "iaea.reports",
  "coinmarketcap.xmr",
  "github.monero-project",
  "worldbank.org",
  "who.int",
];

const CRITERION_TEMPLATES = [
  "Federal spending decreases by 10% by 2030",
  "Median lifespan increases by 5%",
  "CO2 concentration falls below 400 ppm",
  "Homelessness rate decreases by 20%",
  "Cancer mortality rate falls by 15%",
  "Nuclear capacity increases to 100 GW",
  "GDP growth exceeds 3% annually",
  "Poverty rate falls below 10%",
  "Drug approval rate improves by 25%",
  "Open-source contributions exceed 1M per month",
];

function hexKey() {
  return Array.from({ length: 64 }, () =>
    Math.floor(Math.random() * 16).toString(16)
  ).join("");
}

function xmrAmount(min, max) {
  return (Math.random() * (max - min) + min).toFixed(1);
}

function futureDate(minDays, maxDays) {
  const d = new Date();
  d.setDate(d.getDate() + Math.floor(Math.random() * (maxDays - minDays) + minDays));
  return d.toISOString().slice(0, 10);
}

function pastDate(minDays, maxDays) {
  const d = new Date();
  d.setDate(d.getDate() - Math.floor(Math.random() * (maxDays - minDays) + minDays));
  return d.toISOString().slice(0, 10);
}

function makeQuantTerm() {
  return {
    goal_type: "quantitative",
    criterion: faker.helpers.arrayElement(CRITERION_TEMPLATES),
    data_id: `${faker.helpers.arrayElement(DATA_ID_PREFIXES)}.${faker.word.noun()}`,
    operator: faker.helpers.arrayElement(OPERATORS),
    threshold: faker.number.int({ min: 1, max: 10000 }),
    aggregation: faker.helpers.arrayElement([null, null, "sum", "count", "annual_mean"]),
  };
}

function makeQualTerm() {
  return {
    goal_type: "qualitative",
    criterion: faker.company.catchPhrase(),
  };
}

function makeTerms(goalType) {
  if (goalType === "qualitative") return [makeQualTerm()];
  if (goalType === "hybrid") return [makeQuantTerm(), makeQualTerm()];
  // quantitative: 1 or 2 terms
  const count = faker.helpers.arrayElement([1, 1, 2]);
  return Array.from({ length: count }, makeQuantTerm);
}

function oraclePanel(count) {
  return Array.from({ length: count }, () => ({
    pubkey: hexKey(),
    role: faker.helpers.arrayElement(ORACLE_ROLES),
    fee: xmrAmount(1, 100),
  }));
}

function attestations(oraclePanel, quorum, state) {
  if (!["REDEEMABLE", "SETTLED"].includes(state)) return [];
  return oraclePanel.slice(0, quorum).map((o) => ({
    oracle_pubkey: o.pubkey,
    result: "met",
    attested_at: faker.date.recent({ days: 30 }).toISOString(),
    evidence_sha256: hexKey(),
  }));
}

function makeBond(idx) {
  const state = STATES[idx % STATES.length];
  const goalType = faker.helpers.arrayElement(GOAL_TYPES);
  const oracleCount = faker.number.int({ min: 1, max: 5 });
  const quorum = Math.max(1, Math.floor(oracleCount * 0.6));
  const seriesId = `${faker.string.alpha({ length: 3, casing: "upper" })}-${faker.date.recent({ days: 365 }).getFullYear()}`;
  const serial = faker.number.int({ min: 1, max: 999 });
  const currency = faker.helpers.arrayElement(CURRENCIES);
  const lockMechanism = LOCK_MECHANISMS[currency];
  const chainId = currency === "eth" ? faker.helpers.arrayElement(ETH_CHAIN_IDS) : null;

  const isExpiredLike = ["EXPIRED", "SETTLED", "RECLAIMED"].includes(state);
  const deadlineDays = isExpiredLike ? -30 : faker.number.int({ min: 30, max: 730 });
  const deadline =
    isExpiredLike ? pastDate(30, 180) : futureDate(30, 730);
  const expiry =
    isExpiredLike
      ? pastDate(10, 29)
      : futureDate(deadlineDays + 14, deadlineDays + 90);

  const panel = oraclePanel(oracleCount);

  const bond = {
    file: `${seriesId}-${String(serial).padStart(4, "0")}.bond`,
    series_id: seriesId,
    serial,
    state,
    currency,
    lock_mechanism: lockMechanism,
    chain_id: chainId,
    title: faker.company.catchPhrase(),
    description: faker.lorem.sentences(2),
    goal_type: goalType,
    deadline,
    expiry,
    grace_days: faker.helpers.arrayElement([30, 60, 90]),
    amount: xmrAmount(1, 10000),
    alg_epoch: 0,
    holder_pubkey: state !== "DRAFT" ? hexKey() : null,
    oracle_panel: panel,
    quorum,
    challenge_days: faker.helpers.arrayElement([3, 7, 14, 30]),
    created_at: faker.date.past({ years: 1 }).toISOString(),
    issued_at: state !== "DRAFT" ? faker.date.past({ years: 1 }).toISOString() : null,
    accepted_at:
      state !== "DRAFT" ? faker.date.past({ years: 1 }).toISOString() : null,
    attestations: attestations(panel, quorum, state),
  };

  if (state === "SETTLED") {
    bond.settled_at = faker.date.recent({ days: 30 }).toISOString();
  }
  if (state === "RECLAIMED") {
    bond.reclaimed_at = faker.date.recent({ days: 30 }).toISOString();
  }

  const terms = makeTerms(goalType);
  bond.terms = terms;
  if (terms.length > 1) {
    bond.term_aggregation = faker.helpers.arrayElement(["AND", "OR"]);
  }

  if (["REDEEMABLE", "SETTLED"].includes(state)) {
    bond.s_met = hexKey() + hexKey();
  }

  return bond;
}

const bonds = Array.from({ length: COUNT }, (_, i) => makeBond(i));
process.stdout.write(JSON.stringify({ bonds }, null, 2) + "\n");
