
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
        let history = data[command.i];
        if (history === undefined || history[timestamp] === undefined) {
          const pre = document.createElement('pre');
          pre.innerHTML = 'Oops, no data available for this timestamp';
          details.appendChild(pre);
          return;
        }
        const d = history[timestamp];
        h3 = document.createElement('h3');
        h3.innerHTML = 'timestamp';
        const p_command = document.createElement('p');
        p_command.innerHTML = 'Command:';
        const p_pre = document.createElement('pre');
        p_pre.innerHTML = d['c'];
        details.append(h3, p_command, p_pre);

        if (d['x'] !== undefined) {
          const p_exit = document.createElement('p');
          p_exit.innerHTML = `Exit Code: ${d['x']}`;
          details.appendChild(p_exit);
        }
        if (d['t'] !== undefined) {
          const p_timeout = document.createElement('p');
          p_timeout.innerHTML = `Timeout: ${d['t']} seconds`;
          details.appendChild(p_timeout);
        }

        if (d['m'] !== undefined) {
          const p_message = document.createElement('p');
          p_message.innerHTML = 'Error message:';
          const pre = document.createElement('pre');
          pre.innerHTML = d['m'];
          details.append(p_message, pre);
        }

        if (d['o'] !== undefined) {
          const p_output = document.createElement('p');
          p_output.innerHTML = 'Output:';
          const pre = document.createElement('pre');
          pre.innerHTML = d['o'];
          details.append(p_output, pre);
        }
        if (d['e'] !== undefined) {
          const p_error = document.createElement('p');
          p_error.innerHTML = 'Error:';
          const pre = document.createElement('pre');
          pre.innerHTML = d['e'];
          details.append(p_error, pre);
        }
      }
      bean.addEventListener('click', async function() {
        const id = command.i;
        const details = document.getElementById(id);
        const toshow = details.children.length === 0 || details.getAttribute('data-timestamp') !== e.t;
        document.querySelectorAll('.details').forEach(c => {
          c.innerHTML = '';
        });
        if (!toshow) {
          return;
        }
        if (data[command.i] === undefined) {
          details.innerHTML = '';
          details.setAttribute('data-timestamp', e.t);
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
      });
      bar.appendChild(bean);
    }
    div.appendChild(bar);
    const details = document.createElement('div');
    details.classList.add('details');
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
