const {useState, useEffect} = React;

function Cooldown({key, onChange, onRemove, displayName, cooldown, groupName}) {
    return <li key={key}>
        <input
            type="text"
            value={displayName}
            placeholder="{spell:70940} Name"
            onChange={event => onChange({ displayName: event.currentTarget.value, cooldown, groupName })}
        />
        <input
            type="number"
            value={cooldown}
            onChange={event => onChange({ displayName, cooldown: event.currentTarget.value, groupName })}
        />
        <input
            type="text"
            value={groupName}
            placeholder="Group Name"
            onChange={event => onChange({ displayName, cooldown, groupName: event.currentTarget.value })}
        />
        <a
            href="javascript: void(0)"
            onClick={_ => onRemove(key)}
        >Remove</a>
    </li>;
}

function emptyCooldown() {
    return {
        'displayName': '',
        'cooldown': 120,
        'groupName': ''
    };
}

function immutableRemove(array, index) {
    return array.filter((_, otherIndex) => index != otherIndex);
}

function immutableUpdate(array, index, value) {
    return array.map((otherValue, otherIndex) => {
        if (index == otherIndex) {
            return value;
        } else {
            return otherValue;
        }
    });
}

function RaidSetup({ cooldowns, onChange }) {
    return <div className="grid-raid-setup">
        Available Cooldowns:
        <a
            href="javascript: void(0)"
            onClick={_ => onChange({ cooldowns: cooldowns.concat([ emptyCooldown() ]) })}
        >Add</a>
        <ul>
            {cooldowns.map(({ displayName, cooldown, groupName }, index) =>
                <Cooldown
                    key={index}
                    onChange={updated => onChange({ cooldowns: immutableUpdate(cooldowns, index, updated) })}
                    onRemove={_ => onChange({ cooldowns: immutableRemove(cooldowns, index) })}
                    displayName={displayName}
                    cooldown={cooldown}
                    groupName={groupName}
                />
            )}
        </ul>
    </div>;
}

function Template({ template, onChange, isLoading }) {
    let [content, setContent] = useState(template);
    let timerId = null;

    useEffect(() => {
        if (timerId) clearTimeout(timerId)
        timerId = setTimeout(() => onChange({ template: content }), 3000);

        return () => { clearTimeout(timerId) }
    }, [content]);

    return <>
        <textarea
            className="grid-left note-template"
            onInput={event => setContent(event.currentTarget.value)}
            value={content}
        />
    </>;
}

function NoteOutput({ value }) {
    return <>
        <textarea
            className="grid-right note-output"
            value={value}
            readOnly
        />
    </>;
}

function useLocalStorageState(name, initialValue) {
    let stored = localStorage.getItem(name);
    let parsed = stored && JSON.parse(stored);
    let [value, setValue] = useState(parsed || initialValue);

    useEffect(() => {
        if (!value)
            localStorage.removeItem(name);
        else
            localStorage.setItem(name, JSON.stringify(value));
    }, [value]);

    return [value, setValue];
}

function payload({ template, cooldowns }) {
    return {
        template: template,
        cooldowns: cooldowns.map(cooldown => {
            return {
                'displayName': cooldown.displayName,
                'cooldown': parseFloat(cooldown.cooldown),
                'groupNames': cooldown.groupName.split(',').map(groupName => groupName.trim())
            }
        })
    }
}

function Main() {
    let [cooldowns, setCooldowns] = useLocalStorageState('cooldowns', [ emptyCooldown() ]);
    let [template, setTemplate] = useLocalStorageState('template', '');
    let [output, setOutput] = useState('');
    let [isLoading, setLoading] = useState(false);

    useEffect(() => {
        let controller = new AbortController();
        let promise = fetch(
            '/solve',
            {
                'method': 'POST',
                'signal': controller.signal,
                'headers': {
                    'Content-Type': 'application/json'
                },
                'body': JSON.stringify(payload({ template, cooldowns }))
            }
        );

        promise
            .then(response => response.json())
            .then(body => {
                if (body.error)
                    alert(body.error);
                else
                    setOutput(body.output)
            })
            .then(_ => setLoading(false));

        setLoading(true);
        return () => controller.abort();
    }, [template, cooldowns]);

    return <div className="grid-container grid-main">
        <RaidSetup
            cooldowns={cooldowns}
            onChange={({ cooldowns }) => { setCooldowns(cooldowns) }}
        />
        <Template
            template={template}
            onChange={({ template }) => setTemplate(template) }
        />
        <NoteOutput
            value={output}
            isLoading={isLoading}
        />
    </div>;
}

ReactDOM.render(
    <Main />,
    document.querySelector('#root')
);
