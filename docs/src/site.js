const root = document.documentElement;
const toast = document.querySelector(".toast");
let toastTimer;

const notify = (message) => {
  toast.textContent = message;
  toast.classList.add("visible");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toast.classList.remove("visible"), 1800);
};
const copyText = async (value) => {
  if (!navigator.clipboard) throw new Error("Clipboard requires a secure context");
  await navigator.clipboard.writeText(value);
};
const toggleTheme = () => {
  const theme = root.dataset.theme === "dark" ? "light" : "dark";
  root.dataset.theme = theme;
  try { localStorage.setItem("vole-theme", theme); } catch {}
};
document.querySelector("[data-theme-toggle]").addEventListener("click", toggleTheme);

const menu = document.querySelector("[data-menu-toggle]");
const sidebar = document.querySelector(".sidebar");
const closeMenu = () => {
  sidebar.classList.remove("is-open");
  menu.setAttribute("aria-expanded", "false");
  menu.setAttribute("aria-label", "展开导航");
};
menu.addEventListener("click", () => {
  const open = sidebar.classList.toggle("is-open");
  menu.setAttribute("aria-expanded", String(open));
  menu.setAttribute("aria-label", open ? "关闭导航" : "展开导航");
});
document.addEventListener("click", (event) => {
  if (!sidebar.contains(event.target) && !menu.contains(event.target)) closeMenu();
});
matchMedia("(min-width: 55rem)").addEventListener("change", closeMenu);
sidebar.addEventListener("click", (event) => {
  if (event.target instanceof Element && event.target.closest("a")) closeMenu();
});

// Preserve the document scroller so opening a modal does not move sticky navigation.
const lockDialog = (dialog) => {
  const canScroll = (delta) => dialog.scrollHeight > dialog.clientHeight &&
    !(delta < 0 && dialog.scrollTop <= 0) &&
    !(delta > 0 && dialog.scrollTop + dialog.clientHeight >= dialog.scrollHeight - 1);
  dialog.addEventListener("wheel", (event) => {
    if (!canScroll(event.deltaY)) event.preventDefault();
  }, { passive: false });
  let touchY = 0;
  dialog.addEventListener("touchstart", (event) => { touchY = event.touches[0].clientY; }, { passive: true });
  dialog.addEventListener("touchmove", (event) => {
    const nextY = event.touches[0].clientY;
    const delta = touchY - nextY;
    touchY = nextY;
    if (!canScroll(delta)) event.preventDefault();
  }, { passive: false });
  dialog.addEventListener("keydown", (event) => {
    if (event.target.closest("input, textarea, select, [contenteditable]")) return;
    if ([" ", "PageDown", "PageUp", "Home", "End"].includes(event.key)) {
      event.preventDefault();
      if (event.key === "Home") dialog.scrollTop = 0;
      else if (event.key === "End") dialog.scrollTop = dialog.scrollHeight;
      else dialog.scrollTop += (event.key === "PageUp" || event.shiftKey ? -1 : 1) * dialog.clientHeight * .8;
    }
  });
};

const search = document.querySelector(".search-dialog");
const input = search.querySelector("input");
const status = search.querySelector(".search-status");
const results = search.querySelector(".search-results");
let indexPromise;
let searchVersion = 0;
lockDialog(search);
search.addEventListener("click", (event) => {
  const rect = search.getBoundingClientRect();
  if (event.target === search && (event.clientX < rect.left || event.clientX > rect.right ||
      event.clientY < rect.top || event.clientY > rect.bottom)) search.close();
});
const loadIndex = () => indexPromise ??= fetch("/search-index.json")
  .then((response) => {
    if (!response.ok) throw new Error("Search index unavailable");
    return response.json();
  }).catch((error) => { indexPromise = undefined; throw error; });
const performSearch = async () => {
  const version = ++searchVersion;
  const query = input.value.trim().toLocaleLowerCase();
  results.replaceChildren();
  if (!query) { status.textContent = "输入关键词搜索文档"; return; }
  status.textContent = "正在搜索…";
  try {
    const index = await loadIndex();
    if (version !== searchVersion) return;
    const terms = query.split(/\s+/);
    const matches = index.map((entry) => {
      const title = entry.title.toLocaleLowerCase();
      const text = `${title} ${entry.section} ${entry.category_label || ""} ${entry.text}`.toLocaleLowerCase();
      return { entry, score: terms.every((term) => text.includes(term))
        ? terms.reduce((score, term) => score + (title.includes(term) ? 10 : 1), 0) : 0 };
    }).filter((item) => item.score > 0).sort((a, b) => b.score - a.score).slice(0, 20);
    status.textContent = matches.length ? `找到 ${matches.length} 条结果` : "没有匹配的文档";
    for (const { entry } of matches) {
      const item = document.createElement("li");
      const link = document.createElement("a");
      const section = document.createElement("small");
      const title = document.createElement("strong");
      const excerpt = document.createElement("p");
      link.href = entry.path;
      section.textContent = [entry.section, entry.category_label].filter(Boolean).join(" / ");
      title.textContent = entry.title;
      const position = Math.max(0, entry.text.toLocaleLowerCase().indexOf(terms[0]) - 30);
      excerpt.textContent = entry.text.slice(position, position + 130);
      link.append(section, title, excerpt);
      item.append(link);
      results.append(item);
    }
  } catch {
    if (version === searchVersion) status.textContent = "搜索暂时不可用，请重试";
  }
};
const openSearch = () => {
  if (search.open) return;
  search.showModal();
  input.focus();
  void performSearch();
};
document.querySelector("[data-search-open]").addEventListener("click", openSearch);
input.addEventListener("input", performSearch);
search.addEventListener("keydown", (event) => {
  const links = [...results.querySelectorAll("a")];
  const current = links.indexOf(document.activeElement);
  if (event.key === "ArrowDown") {
    event.preventDefault();
    links[Math.min(current + 1, links.length - 1)]?.focus();
  } else if (event.key === "ArrowUp") {
    event.preventDefault();
    if (current <= 0) input.focus(); else links[current - 1].focus();
  } else if (event.key === "Enter" && document.activeElement === input && links.length) {
    event.preventDefault(); links[0].click();
  }
});
search.addEventListener("close", () => { searchVersion++; });

document.addEventListener("keydown", (event) => {
  const editing = event.target instanceof Element && event.target.closest("input, textarea, select, [contenteditable]");
  if (event.key === "Escape") closeMenu();
  if (editing || event.altKey || event.ctrlKey || event.metaKey || document.querySelector("dialog[open]")) return;
  if (event.key === "/") { event.preventDefault(); openSearch(); }
  if (event.key.toLowerCase() === "d") { event.preventDefault(); toggleTheme(); }
});

for (const pre of document.querySelectorAll("pre")) {
  const button = document.createElement("button");
  const icon = document.createElement("img");
  icon.src = "/icons/copy.svg";
  icon.alt = "";
  button.type = "button";
  button.className = "code-copy";
  button.setAttribute("aria-label", "复制代码");
  button.title = "复制代码";
  button.append(icon);
  const code = pre.querySelector("code")?.textContent ?? pre.textContent;
  button.addEventListener("click", async () => {
    button.disabled = true;
    try {
      await copyText(code);
      icon.src = "/icons/copy-check.svg";
      button.setAttribute("aria-label", "已复制");
      button.title = "已复制";
      notify("代码已复制");
    } catch { notify("复制失败，请手动复制"); }
    finally {
      button.disabled = false;
      setTimeout(() => {
        icon.src = "/icons/copy.svg";
        button.setAttribute("aria-label", "复制代码");
        button.title = "复制代码";
      }, 1800);
    }
  });
  pre.append(button);
}

document.querySelector("[data-copy-document]")?.addEventListener("click", async (event) => {
  const button = event.currentTarget;
  button.disabled = true;
  try {
    const response = await fetch(button.dataset.copyDocument);
    if (!response.ok) throw new Error("Markdown unavailable");
    await copyText(await response.text());
    notify("Markdown 已复制");
  } catch { notify("复制失败，请重试"); }
  finally { button.disabled = false; }
});

const preview = document.querySelector(".image-preview");
lockDialog(preview);
let closingPreview = false;
const reduceMotion = matchMedia("(prefers-reduced-motion: reduce)");
const closePreview = async () => {
  if (!preview.open || closingPreview) return;
  closingPreview = true;
  if (!reduceMotion.matches) {
    await preview.animate([{ opacity: 1 }, { opacity: 0 }], { duration: 140 }).finished.catch(() => {});
  }
  preview.close();
  closingPreview = false;
};
preview.addEventListener("click", closePreview);
preview.addEventListener("cancel", (event) => { event.preventDefault(); void closePreview(); });
preview.addEventListener("wheel", (event) => { event.preventDefault(); void closePreview(); }, { passive: false });
for (const image of document.querySelectorAll("img[data-zoom]")) {
  const open = () => {
    if (preview.open) return;
    const target = preview.querySelector("img");
    target.src = image.currentSrc || image.src;
    target.alt = image.alt;
    preview.showModal();
    if (!reduceMotion.matches) preview.animate([{ opacity: 0 }, { opacity: 1 }], { duration: 160 });
  };
  image.tabIndex = 0;
  image.setAttribute("role", "button");
  image.setAttribute("aria-label", `放大图片：${image.alt}`);
  image.addEventListener("click", open);
  image.addEventListener("keydown", (event) => {
    if (event.key === "Enter" || event.key === " ") { event.preventDefault(); open(); }
  });
}

const tocLinks = [...document.querySelectorAll(".toc a")];
const prose = document.querySelector(".prose");
if (tocLinks.length && prose) {
  const headings = [...prose.querySelectorAll("h1, h2, h3, h4, h5, h6")];
  const sections = tocLinks.flatMap((link) => {
    const heading = document.getElementById(decodeURIComponent(link.hash.slice(1)));
    const index = headings.indexOf(heading);
    if (index < 0) return [];
    const level = Number(heading.tagName.slice(1));
    // A section includes its subsections and ends at the next peer or ancestor.
    const end = headings.slice(index + 1).find((next) => Number(next.tagName.slice(1)) <= level);
    return [{ link, heading, end }];
  });
  const header = document.querySelector(".site-header");
  let frame = 0;
  const updateToc = () => {
    frame = 0;
    const top = Math.max(0, header?.getBoundingClientRect().bottom ?? 0);
    const bottom = window.innerHeight;
    const contentBottom = prose.getBoundingClientRect().bottom;
    for (const { link, heading, end } of sections) {
      const sectionTop = heading.getBoundingClientRect().top;
      const sectionBottom = end ? end.getBoundingClientRect().top : contentBottom;
      link.classList.toggle("active", bottom > top && sectionTop < bottom && sectionBottom > top);
    }
  };
  const scheduleTocUpdate = () => {
    if (!frame) frame = requestAnimationFrame(updateToc);
  };
  window.addEventListener("scroll", scheduleTocUpdate, { passive: true });
  window.addEventListener("resize", scheduleTocUpdate, { passive: true });
  const resizeObserver = new ResizeObserver(scheduleTocUpdate);
  resizeObserver.observe(prose);
  if (header) resizeObserver.observe(header);
  scheduleTocUpdate();
}
