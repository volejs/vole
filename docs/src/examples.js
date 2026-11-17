const controls = document.querySelector(".examples-controls");
const input = document.querySelector("#example-search");
const buttons = [...document.querySelectorAll("[data-example-category]")];
const items = [...document.querySelectorAll(".example-entry")];
const groups = [...document.querySelectorAll("[data-example-group]")];
const status = document.querySelector(".examples-status");
const empty = document.querySelector(".examples-empty");
const categories = new Set(buttons.map((button) => button.dataset.exampleCategory));
const initial = new URL(location.href);
let category = categories.has(initial.searchParams.get("category")) ? initial.searchParams.get("category") : "";
input.value = initial.searchParams.get("q") || "";
controls.hidden = false;
controls.addEventListener("submit", (event) => event.preventDefault());

const filter = () => {
  const terms = input.value.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  let count = 0;
  for (const item of items) {
    const matches = (!category || item.dataset.category === category) &&
      terms.every((term) => item.dataset.search.toLocaleLowerCase().includes(term));
    item.hidden = !matches;
    if (matches) count++;
  }
  for (const group of groups) group.hidden = !group.querySelector(".example-entry:not([hidden])");
  for (const button of buttons) button.setAttribute("aria-pressed", String(button.dataset.exampleCategory === category));
  status.textContent = count + " examples";
  empty.hidden = count !== 0;
};
const update = () => {
  filter();
  const url = new URL(location.href);
  if (category) url.searchParams.set("category", category); else url.searchParams.delete("category");
  if (input.value.trim()) url.searchParams.set("q", input.value.trim()); else url.searchParams.delete("q");
  history.replaceState(null, "", url);
};
input.addEventListener("input", update);
for (const button of buttons) button.addEventListener("click", () => { category = button.dataset.exampleCategory; update(); });
window.addEventListener("popstate", () => {
  const url = new URL(location.href);
  category = categories.has(url.searchParams.get("category")) ? url.searchParams.get("category") : "";
  input.value = url.searchParams.get("q") || "";
  filter();
});
filter();
