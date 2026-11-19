"use strict";
/* Docs language tools — local reference data (#2484).
 *
 * Small, deliberately modest lists: enough to prove the local engine works
 * offline. Anything bigger belongs behind a server endpoint (out of scope). */

(function () {
  var NS = window.DocsLang;
  if (!NS) return;

  NS.LANGUAGES = [
    ["en", "docs-lang-lang-en", "English"],
    ["pt", "docs-lang-lang-pt", "Portuguese"],
    ["es", "docs-lang-lang-es", "Spanish"],
    ["fr", "docs-lang-lang-fr", "French"],
    ["de", "docs-lang-lang-de", "German"],
    ["ja", "docs-lang-lang-ja", "Japanese"],
    ["ko", "docs-lang-lang-ko", "Korean"],
    ["zh", "docs-lang-lang-zh", "Chinese"]
  ];

  // Common English words + a few domain-neutral document terms. Split from a
  // compact source so the file stays readable and below the size limit.
  var CORE = (
    "a about above across after again against all also am an and another any are as at be because been before being below between both but by can cannot could did do does doing down during each either else enough even ever every few first for from further get give had has have having he her here hers herself him himself his how i if in into is it its itself just know last less let like made make many may me might more most much must my myself no nor not now of off on once only or other others our ours ourselves out over own same she should so some such than that the their theirs them themselves then there these they this those through to too under until up upon us use used very was we were what when where which while who whom why will with would you your yours yourself yourselves " +
    "able account address agreement already amount annual answer application area arrive article ask associated available away back bad base basic begin beginning best better big body book bring build business call care case change children clear client close collect color come common company complete concern condition connect consider contact contain continue control cost create current customer daily date day decision define department design develop different direct document does done due early easy effect email employee end English enough enter entire environment equal error especially event example expect experience explain express extent face fact factor fail family far field file final find finish first follow form forward found free full function future general goal good great group growth guide hand handle hard head health help high history hold home hour house idea identify imagine improve include increase indeed indicate individual industry information instead interest internal issue item job keep key kind large last late learn leave left legal level light limit line list local long look low machine main make manage market material matter may mean measure meet member mention message method middle might mile million minute model moment money month more morning move name national necessary need network never new next night none north note number object occur offer office often okay old once one only open operation opportunity option order organization original other output overall owner paper part particular partner pass past people period person plan point policy possible power practice prepare present press pretty previous price private problem process program project proper prove provide public purpose quality question quick quite reach read ready real reason receive record reduce refer reference region related relation remain remember report request require research result return review right role room rule run safe sales same save say second section security see seem sell send sense service several shall short show side sign simple since single size small social software someone something sometimes south space speak special start state statement station stop store story strategy strong study stuff subject submit success such suggest support sure system table take talk task team tell term test text than thank think third this time today together tomorrow too total touch toward town track trade train travel treat try turn type understand unit until update use user usually various very view wait want war watch way week weight welcome well west what whatever when where whether which while white who whole why wide wife will win word work world worry would write year yes yet young your"
  ).split(/\s+/).filter(Boolean);

  NS.BASE_WORDS = CORE;

  NS.DOMAIN_TERMS = {
    legal: "amicus curiae assumption of risk best interests consequent consequential damages consideration covenant deed defendant deponent estoppel evidence ex gratia fiduciary force majeure garnishment hearsay hereinafter heretofore indemnification indemnify indemnitee injunction jurisdiction leaseholder legal entity liquidated damages litigation mortgagee novation offeror offeree ordinance plaintiff prima facie probate promissory rescission respondent restitution riparian settlement stamina statute stipulation subpoena substantive testamentary tortfeasor trustee unlimited vested waiver whereas writ".split(/\s+/),
    medical: "administer adverse anesthesia anomaly antibiotics asymptomatic biopsy cardiovascular catheter cerebral clinical congenital contraindication CPR diagnosis differential dilation discharge dosage dyspnea edema emergency endocrine epidemiology etiology exacerbation fatality feasible fever fraction fracture glycolysis hematology hemodynamic hepatic histology hormone hypertension icteric idiopathic immunization incision infusion inhalation inpatient intensive ischemia lesion lumbar malignancy manifest metastasis microbiology myocardial nausea neonatal neutropenia neurological obstetric outpatient overdose oxygenation palliative pathology pediatric perfusion placebo platelet pneumonia prophylaxis pruritus pulmonary radiation radiology resuscitation sepsis symptomatic systolic tachycardia telemetry therapeutic thrombosis thrombocytopenia treatment triage tumor vaccination vasectomy venous vital volatility wound".split(/\s+/),
    financial: "accrual accretion acquisition amortization annuity arbitrage asset-backed audit balance basis points benchmark bond broker capital capex cash flow collateral commercial commitment compliance compound credit debut debenture default derivative dilution discount dividend earnings EBITDA equity escrow exchange exposure face value fiscal flow forecast foreclosure fund hedge hedging income indenture index inflation interest inventory investment issuance levy liability liquidity loan maturity money market mortgage mutual net asset note offering par value portfolio premium principal proceeds profitability projection promissory quarterly quote ratio receivable redemption refinance regress repurchase reserve return revenue risk premium ROI securitization settlement share spread stakeholder suspense tax-deferred tender term treasury underwriter valuation volatility yield".split(/\s+/),
    public: "acquisition appropriation assembly audit authority ballot budget bylaw census charter city council civil service code committee compliance comptroller constituency constitution contract conversion council curator due process election enactment enforcement entity enterprise executive expenditure facilitation federal fiscal gazette grant home rule hearing immunity initiative inspector ordinance ombudsman open meeting ordinance petition planning plebiscite police powers proclamation procurement property public hearing quorum recall referendum regulation report resolution revenue rulemaking rural sanction school board sheriff statute statutory strategic suffrage survey taxpayer territorial transparency treasury unincorporated urban zoning".split(/\s+/)
  };

  NS.THESAURUS = {
    "help": ["assist", "aid", "support", "help out"],
    "use": ["employ", "apply", "utilize", "make use of"],
    "begin": ["start", "commence", "initiate", "set out"],
    "end": ["finish", "conclude", "complete", "terminate"],
    "change": ["modify", "adjust", "alter", "revise"],
    "show": ["demonstrate", "indicate", "display", "illustrate"],
    "important": ["significant", "essential", "critical", "key"],
    "big": ["large", "substantial", "considerable", "major"],
    "small": ["modest", "minor", "compact", "limited"],
    "good": ["excellent", "satisfactory", "beneficial", "sound"],
    "bad": ["poor", "inadequate", "unfavourable", "flawed"],
    "happy": ["content", "pleased", "delighted", "joyful"],
    "sad": ["unhappy", "sorrowful", "melancholy", "downcast"],
    "very": ["extremely", "highly", "particularly", "exceptionally"],
    "method": ["approach", "procedure", "technique", "process"],
    "say": ["state", "remark", "indicate", "explain"],
    "get": ["obtain", "receive", "acquire", "secure"],
    "quick": ["rapid", "swift", "prompt", "fast"],
    "clear": ["explicit", "unambiguous", "lucid", "plain"]
  };

  NS.STYLE_GROUPS = [
    ["plain", "docs-lang-style-plain", "Plain language"],
    ["inclusive", "docs-lang-style-inclusive", "Inclusive language"],
    ["passive", "docs-lang-style-passive", "Passive voice"],
    ["punctuation", "docs-lang-style-punctuation", "Punctuation"]
  ];

  // A null replacement is guidance only: it stays reviewable and has no silent
  // Accept action (handled by the panels module).
  NS.STYLE_RULES = [
    { group: "plain", pattern: /\butilize\b/gi, replacement: "use" },
    { group: "plain", pattern: /\bin order to\b/gi, replacement: "to" },
    { group: "plain", pattern: /\bprior to\b/gi, replacement: "before" },
    { group: "plain", pattern: /\bsubsequent to\b/gi, replacement: "after" },
    { group: "plain", pattern: /\baforementioned\b/gi, replacement: "this" },
    { group: "plain", pattern: /\bcommence\b/gi, replacement: "start" },
    { group: "plain", pattern: /\bterminate\b/gi, replacement: "end" },
    { group: "plain", pattern: /\bapproximately\b/gi, replacement: "about" },
    { group: "plain", pattern: /\bsufficient\b/gi, replacement: "enough" },
    { group: "plain", pattern: /\bnumerous\b/gi, replacement: "many" },
    { group: "plain", pattern: /\bendeavour\b|\bendeavor\b/gi, replacement: "try" },
    { group: "plain", pattern: /\btherefore\b/gi, replacement: "so" },
    { group: "plain", pattern: /\bfacilitate\b/gi, replacement: "help" },
    { group: "inclusive", pattern: /\bmankind\b/gi, replacement: "humanity" },
    { group: "inclusive", pattern: /\bmanpower\b/gi, replacement: "workforce" },
    { group: "inclusive", pattern: /\bchairman\b/gi, replacement: "chair" },
    { group: "inclusive", pattern: /\bsalesman\b/gi, replacement: "salesperson" },
    { group: "inclusive", pattern: /\bhe or she\b/gi, replacement: "they" },
    { group: "inclusive", pattern: /\bhis or her\b/gi, replacement: "their" },
    { group: "inclusive", pattern: /\bguys\b/gi, replacement: "everyone" },
    { group: "inclusive", pattern: /\bdisabled people\b/gi, replacement: "people with disabilities" },
    { group: "passive", pattern: /\b(?:am|is|are|was|were|be|been|being)\s+(?:\w+ed|known|written|done|made|taken|given|seen|held|found|paid|sent|built|used|called|named|listed|reported|issued|approved|published|provided|required|performed|conducted)\b/gi, replacement: null },
    { group: "punctuation", pattern: / {2,}/g, replacement: " " },
    { group: "punctuation", pattern: /\s+([,.;:!?])/g, replacement: "$1" },
    { group: "punctuation", pattern: /,(\S)/g, replacement: ", $1" },
    { group: "punctuation", pattern: /!{2,}/g, replacement: "!" },
    { group: "punctuation", pattern: /\?{2,}/g, replacement: "?" }
  ];
})();
