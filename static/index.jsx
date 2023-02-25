const {useState, useEffect} = React;

function ButtonLink({ onClick, className, ...props }) {
    return <a
        href="javascript: void(0)"
        className={"text-s text-blue-600 hover:text-blue-900 " + className}
        onClick={onClick}
    >
        {props.children}
    </a>;
}

function Cooldown({key, onChange, onRemove, displayName, cooldown, groupName}) {
    return <tr key={key}>
        <td>
            <input
                type="text"
                value={displayName}
                placeholder="{spell:70940} Name"
                className="form-input rounded"
                onChange={event => onChange({ displayName: event.currentTarget.value, cooldown, groupName })}
            />
        </td>
        <td>
            <input
                type="number"
                value={cooldown}
                placeholder="Cooldown (sec)"
                className="form-input rounded"
                onChange={event => onChange({ displayName, cooldown: event.currentTarget.value, groupName })}
            />
        </td>
        <td>
            <input
                type="text"
                value={groupName}
                placeholder="Group Names"
                className="form-input rounded"
                size={50}
                onChange={event => onChange({ displayName, cooldown, groupName: event.currentTarget.value })}
            />
        </td>
        <td>
            <ButtonLink onClick={_ => onRemove(key)}>[Remove]</ButtonLink>
        </td>
    </tr>;
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
    return <div className="flex-none bg-gray-100 p-1">
        <span className="font-bold">Available Cooldowns:</span>
        <ButtonLink
            className="pl-2"
            onClick={_ => onChange({ cooldowns: cooldowns.concat([ emptyCooldown() ]) })}
        >[Add]</ButtonLink>
        <table className="table-fixed">
            <tbody>
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
            </tbody>
        </table>
    </div>;
}

function Template({ template, onChange, onInput }) {
    let [content, setContent] = useState(template);
    let timerId = null;

    useEffect(() => {
        if (timerId) clearTimeout(timerId)
        timerId = setTimeout(() => onChange({ template: content }), 3000);
        onInput({template: content });

        return () => { clearTimeout(timerId) }
    }, [content]);

    return <>
        <textarea
            className="flex-auto rounded resize-none"
            onInput={event => setContent(event.currentTarget.value)}
            value={content}
        />
    </>;
}

function Spinner() {
    return <svg aria-hidden="true" class="w-8 h-8 mr-2 text-gray-200 animate-spin dark:text-gray-600 fill-blue-600" viewBox="0 0 100 101" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z" fill="currentColor"/>
        <path d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z" fill="currentFill"/>
    </svg>;
}

function NoteOutput({ value, isLoading }) {
    return <div className="flex-auto relative">
        {isLoading && <span className="absolute m-auto inset-1/2"><Spinner /></span>}
        <textarea
            className="h-full w-full rounded resize-none bg-gray-100"
            value={value}
            readOnly
        />
    </div>;
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

    return <div className="flex flex-col space-y-1 h-full">
        <RaidSetup
            cooldowns={cooldowns}
            onChange={({ cooldowns }) => { setCooldowns(cooldowns) }}
        />
        <div className="flex flex-row space-x-1 flex-auto p-1">
            <Template
                template={template}
                onChange={({ template }) => setTemplate(template)}
                onInput={_ => setLoading(true)}
            />
            <NoteOutput
                value={output}
                isLoading={isLoading}
            />
        </div>
    </div>;
}

ReactDOM.render(
    <Main />,
    document.querySelector('#root')
);
