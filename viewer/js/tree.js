// The Loops tab: the loop tree as nested <details>, children built on first open.

const MAX_IDS = 16;

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

/**
 * @param {HTMLElement} root
 * @param {{onJump: (segment: number) => void}} options
 */
export function createTree(root, { onJump }) {
  let view = null;
  let own = [];

  function label(index, node) {
    const summary = el('summary');
    const name = node.loop_name === null ? 'file' : `${node.loop_name} #${node.ordinal}`;
    summary.append(el('strong', null, name));
    if (node.implicit) summary.append(el('span', 'tag', ' (implicit)'));
    if (node.children.length) summary.append(el('span', 'tag', ` · ${node.children.length} nested`));
    const segments = own[index];
    if (segments.length) {
      const ids = el('span', 'ids');
      ids.append(' ');
      for (const s of segments.slice(0, MAX_IDS)) {
        const button = el('button', 'link', view.segments[s].id);
        button.type = 'button';
        button.title = `Segment ${s}`;
        button.addEventListener('click', (event) => {
          // A click on the id must not fold or unfold the node.
          event.preventDefault();
          onJump(s);
        });
        ids.append(button);
      }
      if (segments.length > MAX_IDS) ids.append(el('span', 'tag', `… +${segments.length - MAX_IDS}`));
      summary.append(ids);
    }
    return summary;
  }

  function build(index) {
    const node = view.nodes[index];
    const details = el('details');
    details.append(label(index, node));
    if (node.children.length === 0) {
      details.classList.add('leaf');
      return details;
    }
    let built = false;
    const fill = () => {
      if (built) return;
      built = true;
      const list = el('ul');
      for (const child of node.children) {
        const li = el('li');
        li.append(build(child));
        list.append(li);
      }
      details.append(list);
    };
    details.addEventListener('toggle', () => {
      if (details.open) fill();
    });
    details.fill = fill;
    return details;
  }

  return {
    show(loaded) {
      view = loaded.view;
      own = view.nodes.map(() => []);
      view.segments.forEach((s, i) => {
        if (s.id !== '' && own[s.node]) own[s.node].push(i);
      });
      root.textContent = '';
      const tree = el('div', 'tree');
      let index = 0;
      let details = build(index);
      tree.append(details);
      // Open the root and follow a chain of single children (interchange, group, transaction).
      for (;;) {
        details.open = true;
        details.fill?.();
        const node = view.nodes[index];
        if (node.children.length !== 1) break;
        index = node.children[0];
        details = details.querySelector(':scope > ul > li > details');
        if (!details) break;
      }
      root.append(tree);
    },
    clear() {
      view = null;
      own = [];
      root.textContent = '';
    },
  };
}
