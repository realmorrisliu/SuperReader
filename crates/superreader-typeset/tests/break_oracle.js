const fs = require("fs");
const path = require("path");
const src = fs.readFileSync(
  path.join(__dirname, "../../../spikes/justify/vendor/tex-linebreak/layout.js"),
  "utf8",
);
const createTexLayout = new Function(src + "\nreturn createTexLayout;")();
const tex = createTexLayout();
const input = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const jobs = Array.isArray(input) ? input : [input];
const out = jobs.map((job) => {
  try {
    return { ok: true, breaks: tex.breakLines(job.items, job.lengths, job.opts) };
  } catch (err) {
    return { ok: false, error: err.constructor.name };
  }
});
process.stdout.write(JSON.stringify(Array.isArray(input) ? out : out[0]));
