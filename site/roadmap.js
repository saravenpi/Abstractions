(function () {
  const LIVE = "https://api.github.com/repos/saravenpi/Abstractions/contents/ROADMAP.md?ref=main";
  const SHIPPED = "/roadmap/ROADMAP.md";
  const REPO = "https://github.com/saravenpi/Abstractions/blob/main/";
  const VERSION = /^(\d+(?:\.\d+)+):\s*(.+)$/;
  const STATUS = /^\*\*Status:\*\*\s*(.+?)\.?\s*$/i;
  const HEADING = /^(#{3,6})\s+(.+?)\s*#*\s*$/;
  const ITEM = /^(\s*)([-*+]|\d+[.)])\s+(.*)$/;
  const TASK = /^\[([ xX])\]\s+(.*)$/;
  const DIVIDER = /^\s*\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)*\|?\s*$/;
  const INLINE =
    /(`+)([\s\S]*?[^`])\1(?!`)|\[([^\]]+)\]\(([^)\s]+)\)|\*\*([\s\S]+?)\*\*|\*([^*\s][^*]*?)\*|(https?:\/\/[^\s<>()]*[^\s<>().,;:!?'"])/g;
  const KINDS = { released: "released", "in progress": "progress", planned: "planned", "not yet ordered": "unordered" };
  const LABELS = { released: "Released", progress: "In progress", planned: "Planned", unordered: "Not yet ordered" };

  const timeline = document.getElementById("timeline");
  const note = document.getElementById("road-note");
  const source = document.getElementById("road-source");
  const summary = document.getElementById("summary");
  const filter = document.querySelector(".filter");
  let shown = "";
  let live = false;
  let current = "all";
  let tagged = false;

  function escape(text) {
    return String(text).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  }

  function href(url) {
    if (/^(https?:|mailto:)/i.test(url) || /^[/#]/.test(url)) return url;
    if (/^[a-z][a-z0-9+.-]*:/i.test(url)) return "";
    return REPO + url.replace(/^\.\//, "");
  }

  function inline(text) {
    let out = "";
    let last = 0;
    text.replace(INLINE, function (match, ticks, code, label, url, bold, italic, bare, offset) {
      out += escape(text.slice(last, offset));
      last = offset + match.length;
      if (ticks) out += "<code>" + escape(code.trim()) + "</code>";
      else if (label) {
        const target = href(url);
        out += target ? '<a class="inline" href="' + escape(target) + '">' + inline(label) + "</a>" : inline(label);
      } else if (bold) out += "<strong>" + inline(bold) + "</strong>";
      else if (italic) out += "<em>" + inline(italic) + "</em>";
      else out += '<a class="inline" href="' + escape(bare) + '">' + escape(bare) + "</a>";
      return match;
    });
    return out + escape(text.slice(last));
  }

  function plain(text) {
    return text.replace(/[*`]/g, "").replace(/\[([^\]]+)\]\([^)]*\)/g, "$1");
  }

  function cells(line) {
    return line.trim().replace(/^\|/, "").replace(/\|$/, "").split("|").map((cell) => cell.trim());
  }

  function isTable(lines, i) {
    return /^\s*\|/.test(lines[i]) && i + 1 < lines.length && DIVIDER.test(lines[i + 1]);
  }

  function starts(lines, i) {
    return !lines[i].trim() || HEADING.test(lines[i]) || ITEM.test(lines[i]) || isTable(lines, i);
  }

  function item(text) {
    const task = TASK.exec(text);
    return { task: task ? task[1] !== " " : null, text: task ? task[2] : text, children: [] };
  }

  function list(lines, start, indent) {
    const first = ITEM.exec(lines[start]);
    const node = { type: "list", ordered: /\d/.test(first[2]), start: parseInt(first[2], 10) || 1, items: [] };
    let i = start;
    while (i < lines.length) {
      const line = lines[i];
      const last = node.items[node.items.length - 1];
      if (!line.trim()) {
        let j = i + 1;
        while (j < lines.length && !lines[j].trim()) j++;
        const next = j < lines.length && ITEM.exec(lines[j]);
        if (next && next[1].length >= indent) {
          i = j;
          continue;
        }
        break;
      }
      const match = ITEM.exec(line);
      const depth = match ? match[1].length : line.search(/\S/);
      if (match && depth === indent) {
        node.items.push(item(match[3]));
        i++;
      } else if (match && depth > indent && last) {
        const sub = list(lines, i, depth);
        last.children.push(sub.node);
        i = sub.end;
      } else if (!match && depth > indent && last) {
        last.text += " " + line.trim();
        i++;
      } else break;
    }
    return { node, end: i };
  }

  function blocks(lines) {
    const out = [];
    let i = 0;
    while (i < lines.length) {
      const line = lines[i];
      const heading = HEADING.exec(line);
      if (!line.trim()) i++;
      else if (heading) {
        out.push({ type: "heading", level: Math.min(heading[1].length, 4), text: heading[2] });
        i++;
      } else if (isTable(lines, i)) {
        const head = cells(line);
        const rows = [];
        i += 2;
        while (i < lines.length && /^\s*\|/.test(lines[i])) rows.push(cells(lines[i++]));
        out.push({ type: "table", head, rows });
      } else if (ITEM.test(line)) {
        const result = list(lines, i, ITEM.exec(line)[1].length);
        out.push(result.node);
        i = result.end;
      } else {
        const text = [line.trim()];
        i++;
        while (i < lines.length && !starts(lines, i)) text.push(lines[i++].trim());
        out.push({ type: "paragraph", text: text.join(" ") });
      }
    }
    return out;
  }

  function renderList(node) {
    const tag = node.ordered ? "ol" : "ul";
    const start = node.ordered && node.start !== 1 ? ' start="' + node.start + '"' : "";
    const items = node.items.map(function (entry) {
      const inner = inline(entry.text) + entry.children.map(renderList).join("");
      if (entry.task === null) return "<li>" + inner + "</li>";
      const state = entry.task ? "Done: " : "To do: ";
      return '<li class="task' + (entry.task ? " is-done" : "") + '"><span class="box" aria-hidden="true"></span><span><span class="sr">' + state + "</span>" + inner + "</span></li>";
    });
    return "<" + tag + start + ">" + items.join("") + "</" + tag + ">";
  }

  function renderTable(node) {
    const head = node.head.map((cell) => '<th scope="col">' + inline(cell) + "</th>").join("");
    const rows = node.rows.map(function (row) {
      const tds = node.head.map((name, index) => '<td data-label="' + escape(plain(name)) + '">' + inline(row[index] || "") + "</td>");
      return "<tr>" + tds.join("") + "</tr>";
    });
    return '<div class="scroll"><table class="settings"><thead><tr>' + head + "</tr></thead><tbody>" + rows.join("") + "</tbody></table></div>";
  }

  function render(nodes) {
    return nodes
      .map(function (node) {
        if (node.type === "heading") return "<h" + node.level + ">" + inline(node.text) + "</h" + node.level + ">";
        if (node.type === "paragraph") return "<p>" + inline(node.text) + "</p>";
        if (node.type === "list") return renderList(node);
        return renderTable(node);
      })
      .join("");
  }

  function tasks(nodes) {
    const count = { done: 0, total: 0 };
    function walk(node) {
      node.items.forEach(function (entry) {
        if (entry.task !== null) {
          count.total++;
          if (entry.task) count.done++;
        }
        entry.children.forEach(walk);
      });
    }
    nodes.filter((node) => node.type === "list").forEach(walk);
    return count;
  }

  function gist(nodes) {
    const first = nodes.find((node) => node.type === "paragraph" || node.type === "list");
    if (!first) return "";
    if (first.type === "paragraph") return inline(first.text);
    return first.items.map((entry) => inline(entry.text)).join("; ");
  }

  function sections(text) {
    const out = [];
    let section = null;
    text.split("\n").forEach(function (line) {
      const head = /^##\s+(.+?)\s*$/.exec(line);
      if (head) {
        section = { title: head[1], lines: [] };
        out.push(section);
      } else if (section) section.lines.push(line);
    });
    return out;
  }

  function milestone(section) {
    const match = VERSION.exec(section.title);
    if (!match && !/^later$/i.test(section.title)) return null;
    const at = section.lines.findIndex((line) => STATUS.test(line.trim()));
    const status = at >= 0 ? STATUS.exec(section.lines[at].trim())[1].toLowerCase() : "";
    const lines = at >= 0 ? section.lines.slice(0, at).concat(section.lines.slice(at + 1)) : section.lines;
    const name = match ? match[2] : section.title;
    const nodes = blocks(lines);
    return {
      id: match ? "v" + match[1].replace(/\./g, "-") : "later",
      version: match ? match[1] : "Later",
      name: name.charAt(0).toUpperCase() + name.slice(1),
      kind: KINDS[status] || "planned",
      dated: at >= 0,
      nodes,
      tasks: tasks(nodes),
    };
  }

  function parse(text) {
    const all = sections(text);
    const find = (pattern) => all.find((section) => pattern.test(section.title));
    return {
      setting: find(/^the setting$/i),
      rules: find(/^rules that hold in every version$/i),
      milestones: all.map(milestone).filter(Boolean),
    };
  }

  function group(kind) {
    return kind === "unordered" ? "planned" : kind;
  }

  function meter(count) {
    if (!count.total) return "";
    const cellsHtml = Array.from({ length: count.total }, (_, index) => (index < count.done ? '<i class="on"></i>' : "<i></i>")).join("");
    return '<div class="meter"><span class="cells" aria-hidden="true">' + cellsHtml + '</span><span class="label">' + count.done + " of " + count.total + " done</span></div>";
  }

  function card(stone, open) {
    const body = stone.id + "-body";
    const li = document.createElement("li");
    li.className = "stone is-" + stone.kind;
    li.id = stone.id;
    li.dataset.group = group(stone.kind);
    li.innerHTML =
      '<span class="dot" aria-hidden="true"></span><div class="stone-in"><p class="stone-meta"><a class="ver" href="#' + stone.id + '">' + escape(stone.version) +
      '</a><span class="state">' + LABELS[stone.kind] + '</span></p><div class="stone-main"><h2 class="stone-name">' + escape(stone.name) + "</h2>" +
      '<p class="gist">' + gist(stone.nodes) + "</p>" + meter(stone.tasks) +
      '<button type="button" class="toggle" aria-controls="' + body + '"></button><div class="md" id="' + body + '">' + render(stone.nodes) + "</div></div></div>";
    li.querySelector(".toggle").addEventListener("click", () => expand(li, li.querySelector(".toggle").getAttribute("aria-expanded") !== "true"));
    expand(li, open);
    return li;
  }

  function expand(li, open) {
    const button = li.querySelector(".toggle");
    button.setAttribute("aria-expanded", String(open));
    button.textContent = open ? "Hide details" : "Details";
    li.querySelector(".md").hidden = !open;
    li.querySelector(".gist").hidden = open;
    li.classList.toggle("is-open", open);
  }

  function apply(name) {
    current = name;
    filter.querySelectorAll("button").forEach((button) => button.setAttribute("aria-pressed", String(button.dataset.filter === name)));
    timeline.querySelectorAll(".stone").forEach((li) => (li.hidden = name !== "all" && li.dataset.group !== name));
  }

  function counts(milestones) {
    const total = { released: 0, progress: 0, planned: 0 };
    milestones.forEach((stone) => total[group(stone.kind)]++);
    const parts = [];
    if (total.released) parts.push(total.released + " released");
    if (total.progress) parts.push(total.progress + " in progress");
    if (total.planned) parts.push(total.planned + " planned");
    return parts.join(", ");
  }

  function target() {
    const id = decodeURIComponent(location.hash.slice(1));
    return id && document.getElementById(id) && document.getElementById(id).classList.contains("stone") ? document.getElementById(id) : null;
  }

  function jump() {
    const li = target();
    if (!li) return;
    if (li.hidden) apply("all");
    expand(li, true);
    li.scrollIntoView();
  }

  function intro(data) {
    const setting = document.getElementById("setting");
    const rules = document.getElementById("rules");
    setting.innerHTML = data.setting ? render(blocks(data.setting.lines)) : "";
    if (data.rules) document.getElementById("rules-body").innerHTML = render(blocks(data.rules.lines));
    rules.hidden = !data.rules;
  }

  function released() {
    const arrives = document.getElementById("arrives");
    const hide = function () {
      tagged = true;
      arrives.hidden = true;
    };
    if (Date.now() > Date.parse("2026-11-23")) return hide();
    fetch("https://api.github.com/repos/saravenpi/Abstractions/releases/latest", { headers: { Accept: "application/vnd.github+json" } })
      .then((response) => (response.ok ? response.json() : Promise.reject(response.status)))
      .then(function (release) {
        const tag = /^v(\d+)\.(\d+)\.\d+$/.exec(release.tag_name);
        if (tag && (+tag[1] > 0 || +tag[2] >= 11)) hide();
      })
      .catch(function () {});
  }

  function draw(data) {
    const first = !timeline.children.length;
    const opened = new Set(Array.from(timeline.querySelectorAll(".stone.is-open"), (li) => li.id));
    const fragment = document.createDocumentFragment();
    const lead = data.milestones.find((stone) => stone.kind === "progress") || data.milestones.find((stone) => stone.kind === "planned");
    data.milestones.forEach(function (stone) {
      fragment.append(card(stone, first ? stone === lead : opened.has(stone.id)));
    });
    intro(data);
    document.getElementById("arrives").hidden = tagged || !data.milestones.some((stone) => stone.id === "v0-11" && stone.kind === "progress");
    timeline.replaceChildren(fragment);
    summary.textContent = counts(data.milestones);
    filter.hidden = false;
    note.textContent = "";
    apply(current);
    if (first) jump();
  }

  function show(text, from) {
    const clean = text.replace(/\r\n?/g, "\n").trim();
    if (live || (from === "shipped" && shown)) return;
    if (clean !== shown) {
      const data = parse(clean);
      if (!data.milestones.some((stone) => stone.dated)) return;
      shown = clean;
      draw(data);
    }
    live = from === "live";
    source.innerHTML = live
      ? 'Read live from <a class="inline" href="' + REPO + 'ROADMAP.md">ROADMAP.md on GitHub</a>'
      : 'Copy shipped with this site. <a class="inline" href="' + REPO + 'ROADMAP.md">ROADMAP.md on GitHub</a> may be newer';
  }

  function get(url, headers) {
    return fetch(url, { headers }).then((response) => (response.ok ? response.text() : Promise.reject(response.status)));
  }

  filter.addEventListener("click", function (event) {
    const button = event.target.closest("button[data-filter]");
    if (button) apply(button.dataset.filter);
  });
  window.addEventListener("hashchange", jump);
  note.textContent = "Loading the roadmap.";
  released();

  const shipped = get(SHIPPED).then((text) => show(text, "shipped"));
  const fresh = get(LIVE, { Accept: "application/vnd.github.raw" }).then((text) => show(text, "live"));
  Promise.all([shipped.catch(() => {}), fresh.catch(() => {})]).then(function () {
    if (shown) return;
    note.innerHTML = 'The roadmap could not be loaded. <a class="inline" href="' + REPO + 'ROADMAP.md">Read ROADMAP.md on GitHub</a>.';
  });
})();
