
// Store downloaded data
// key: command id
let data = {};

function processSummary(summary, title) {
  // Update Title
  const status = (summary.nb_err == 0)? '\u{2714}' : '\u{2718}';
  const total = summary.nb_err + summary.nb_ok;
  document.title = `${status} ${summary.nb_ok}/${total} - ${title}`;
  // Update Summary
  const h1 = document.getElementById('summary');
  if (summary.nb_err == 0) {
    h1.classList.add('ok');
    h1.classList.remove('err');
    h1.innerHTML = '\u{2714} All Systems Operational';
  } else {
    h1.classList.add('err');
    h1.classList.remove('ok');
    const plural = (summary.nb_err > 1) ? 's' : '';
    h1.innerHTML = `\u{26A0} ${summary.nb_err} command${plural} failed`;
  }
}

function processCommands(commands) {
  const container = document.getElementById('commands');
  container.innerHTML = '';

  commands.forEach(command => {
    const div = document.createElement('div');
    div.classList.add('command');
    const name = document.createElement('h2');
    name.innerHTML = command.n;
    div.appendChild(name);

    const bar = document.createElement('div');
    bar.classList.add('bar');
    bar.setAttribute('data-id', command.i);
    for (const e of command.e) {
      const bean = document.createElement('div');
      bean.classList.add('bean');
      bean.classList.add(e.e ? 'err': 'ok');
      bean.classList.add(e.k == 'd' ? 'day' : (e.k == 'h' ? 'hour' : 'minute'));
      bean.setAttribute('title', e.t);
      bean.innerHTML = e.v;

      const renderDetails = function (id, timestamp) {
        const details = document.getElementById(id);
        details.innerHTML = '';
        const pre = document.createElement('pre');
        let history = data[command.i];
        if (history === undefined || history[timestamp] === undefined) {
          pre.innerHTML = 'Oops, no data available for this timestamp';
          details.appendChild(pre);
          return;
        }
        pre.innerHTML = JSON.stringify(data[command.i], null, 2);
        details.appendChild(pre);
      }
      bean.addEventListener('click', async function() {
        const id = command.i;
        if (data[command.i] === undefined) {
          const details = document.getElementById(id);
          details.innerHTML = '';
          const pre = document.createElement('pre');
          pre.innerHTML = 'Loading...';
          details.appendChild(pre);

          const requestURL = `${command.i}.json`;
          const request = new Request(requestURL);
          const response = await fetch(request);
          const jsonText = await response.text();

          const json = JSON.parse(jsonText);
          data[command.i] = json['h'];
          renderDetails(command.i, e.t);
        } else {
          renderDetails(command.i, e.t);
        }
        const timestamp = this.getAttribute('title');
        const elem = document.getElementById(id);
        const toshow = elem.classList.contains('hidden');
        document.querySelectorAll('.details').forEach(c => {
          c.classList.add('hidden');
        });
        if (toshow) {
          elem.classList.remove('hidden');
        }
      });
      bar.appendChild(bean);
    }
    div.appendChild(bar);
    const details = document.createElement('div');
    details.classList.add('container_details');
    details.classList.add('hidden');
    details.setAttribute('id', command.i);
    div.appendChild(details);
    container.appendChild(div);
  });
}

async function populate() {
  const requestURL = "main.json";
  const request = new Request(requestURL);
  const response = await fetch(request);
  const mainText = await response.text();

  const main = JSON.parse(mainText);
  processSummary(main.s, main.t);
  processCommands(main.c);
}

document.addEventListener("DOMContentLoaded", function() {
  populate();
});
