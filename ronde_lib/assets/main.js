
function processSummary(summary, title) {
  // Update Title
  const status = (summary.nb_err == 0)? '\u{2714}' : '\u{2718}';
  const total = summary.nb_err + summary.nb_ok;
  document.title = `${status} ${summary.nb_err}/${total} - ${title}`;
  // Update Summary
  const h1 = document.getElementById('summary');
  console.log(h1);
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

async function populate() {
  const requestURL = "main.json";
  const request = new Request(requestURL);
  const response = await fetch(request);
  const mainText = await response.text();

  const main = JSON.parse(mainText);
  processSummary(main.s, main.t);
  //processCommands(main.h);
}

document.addEventListener("DOMContentLoaded", function() {

  populate();


  const beans = document.querySelectorAll('.bean');

  beans.forEach(item => {
    item.addEventListener('click', function() {
      const id = this.getAttribute('data-toggle');
      const elem = document.getElementById(id);
      const toshow = elem.classList.contains('hidden');
      document.querySelectorAll('.details').forEach(c => {
        c.classList.add('hidden');
      });
      if (toshow) {
        elem.classList.remove('hidden');
      }
    });
  });
});
