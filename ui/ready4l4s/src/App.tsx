import { useState } from 'react';
import logo from './l4s.png'
import './App.css';

const doppler_url = "https://raw.githubusercontent.com/cerfcast/dopplers/main/dopplers.json";

function RunTest({ add_result_information }: { add_result_information: (new_status: string) => void }) {
  return (
    <button onClick={do_test}>
      Do a test
    </button>
  );

  function do_test() {
    fetch(doppler_url).then((response: Response) => {
      response.text().then((text) => {
        // We know that we were able to download the doppler information.
        // Let's use it.
        const doppler_object = JSON.parse(text);
        const endpoint_name = doppler_object['dopplers'][0]['endpoints'][0]['name'];
        const endpoint_hostname = doppler_object['dopplers'][0]['endpoints'][0]['hostname'];
        const connectivity_port = doppler_object['dopplers'][0]['endpoints'][0]['connectivity'];
        const ready_port = doppler_object['dopplers'][0]['endpoints'][0]['ready'];

        const connectivity_url = "https://" + endpoint_hostname + ":" + connectivity_port + "/";
        const ready_url = "https://" + endpoint_hostname + ":" + ready_port + "/";
        add_result_information(`Beginning test to the ${endpoint_name} testing endpoint...`)
        console.log(`Using ${connectivity_url} as the basis for the basic connectivity determination.`)
        console.log(`Using ${ready_url} as the basis for the L4S readiness determination.`)

        fetch(connectivity_url).then((connectivity_check_response: Response) => {
          if (connectivity_check_response.status === 200) {
            add_result_information(`Basic connectivity check has established a baseline necessary for test validity.`)
          } else {
            add_result_information(`Could not establish basic connectivity necessary to perform the test.`);
          }
        }, (connectivity_check_error) => {
          add_result_information(`Could not establish basic connectivity necessary to perform the test.`);
        })

        let controller = new AbortController();
        const signal = controller.signal;

        const timeouter = new Promise((resolve: (value: number) => void, reject: (reason: any) => void) => {
          setTimeout(() => { resolve(0) }, 5000)
        });

        timeouter.then((ignored: number) => {
          console.log("Aborting!");
          controller.abort();
        });

        fetch(ready_url, { signal }).then((readiness_check_response: Response) => {
          if (readiness_check_response.status === 200) {
            add_result_information(`Readiness connectivity check established that the connection is L4S ready.`);
          } else {
            add_result_information(`Readiness connectivity check established that the connection may not be L4S ready.`);
            console.log(`Readiness check error: ${readiness_check_response.status}`)
          }
        }, (readiness_check_error) => {
          add_result_information(`Readiness connectivity check established that the connection may not be L4S ready.`);
          console.log(`Readiness check error: ${readiness_check_error}`)
        })
      }, (failure: any) => {
        add_result_information(`There was an error fetching test configuration information: ${failure}`);
        console.log(`Readiness check error: ${failure}`)
      })
    }, (failure: any) => {
      add_result_information(`There was an error fetching test configuration information: ${failure}`);
      console.log(`Readiness check error: ${failure}`)
    })
  }
}

function HaveDialog({ value }: { value: string }) {
  return (
    <header className="Results-header" dangerouslySetInnerHTML={{ __html: value }}>
    </header>
  );
}

function App() {
  const [dialogue_value, set_dialogue_value] = useState<string>("")

  function append_status(msg: string): void {
    set_dialogue_value(existing => existing + "<br>" + msg);
  }

  return (
    <div className="App">
      <header className="App-header">
        <div className="Question-div">
          <span className="Question-text">Are You Ready For</span>
          <span className="Question-text">
            <img className="Question-logo" src={logo}></img>
          </span>
          <span className="Question-text">?</span>
        </div>
      </header>

      <HaveDialog value={dialogue_value}></HaveDialog>
      <RunTest add_result_information={append_status}></RunTest>
    </div>
  );
}

export default App;
